//! `gRPC` plugin for Legion's ECS.
//!
//! Provides `gRPC` server support to the engine, compatible with the `tonic`
//! crate.

// crate-specific lint exceptions:
//#![allow()]

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::Router;
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_online::grpc::{
    multiplexer_service::{MultiplexableService, MultiplexerService, MultiplexerServiceBuilder},
    HybridServer, Server,
};
use lgn_tracing::warn;
use tonic::transport::NamedService;

/// Label to use for scheduling systems for GRPC Service registration
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum GRPCPluginScheduling {
    StartRpcServer,
}

pub struct GRPCPluginSettings {
    pub grpc_server_addr: SocketAddr,
    pub rest_server_addr: Option<SocketAddr>,
    multiplexer_service_builder: MultiplexerServiceBuilder,
    /// Used only in "hybrid mode"
    router: Option<Arc<Mutex<Router>>>,
}

impl GRPCPluginSettings {
    pub fn new(grpc_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr,
            rest_server_addr: None,
            multiplexer_service_builder: MultiplexerService::builder(),
            router: None,
        }
    }

    pub fn hybrid(grpc_server_addr: SocketAddr, rest_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr,
            rest_server_addr: Some(rest_server_addr),
            multiplexer_service_builder: MultiplexerService::builder(),
            router: None,
        }
    }

    pub fn register_service<S>(&mut self, s: S) -> &mut Self
    where
        S: MultiplexableService + NamedService + Send + Sync + 'static,
    {
        self.multiplexer_service_builder.add_service(s);

        self
    }

    pub fn register_router(&mut self, router: Router) -> &Self {
        self.router = Some(Arc::new(Mutex::new(router)));

        self
    }
}

impl Default for GRPCPluginSettings {
    fn default() -> Self {
        Self::new("[::1]:50051".parse().unwrap())
    }
}

// Provides gRPC server capabilities to the engine.
#[derive(Default)]
pub struct GRPCPlugin {
    hybrid: bool,
}

impl Plugin for GRPCPlugin {
    fn build(&self, app: &mut App) {
        let system = if self.hybrid {
            Self::start_hybrid_server
        } else {
            Self::start_grpc_server
        };

        let system = system
            .exclusive_system()
            .label(GRPCPluginScheduling::StartRpcServer);

        app.init_resource::<GRPCPluginSettings>()
            .add_startup_system_to_stage(StartupStage::PostStartup, system);
    }
}

impl GRPCPlugin {
    pub fn hybrid() -> Self {
        Self { hybrid: true }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn start_grpc_server(
        settings: Res<'_, GRPCPluginSettings>,
        rt: ResMut<'_, lgn_async::TokioAsyncRuntime>,
    ) {
        if let Some(service) = settings.multiplexer_service_builder.build() {
            let server = Server::default().set_listen_address(settings.grpc_server_addr);

            rt.start_detached(async move {
                match server.run(service).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        warn!("gRPC server stopped and no longer listening ({})", e);

                        Err(e)
                    }
                }
            });
        } else {
            warn!("not starting gRPC server as no service was registered");
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn start_hybrid_server(
        settings: Res<'_, GRPCPluginSettings>,
        rt: ResMut<'_, lgn_async::TokioAsyncRuntime>,
    ) {
        if let Some(service) = settings.multiplexer_service_builder.build() {
            if let Some(router) = settings.router.clone() {
                let mut server =
                    HybridServer::default().set_grpc_listen_address(settings.grpc_server_addr);

                if let Some(rest_server_addr) = settings.rest_server_addr {
                    server = server.set_rest_listen_address(rest_server_addr);
                }

                rt.start_detached(async move {
                    match server.run(service, router).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            warn!(
                                "Rest or gRPC server stopped and no longer listening ({})",
                                e
                            );

                            Err(e)
                        }
                    }
                });
            } else {
                warn!("not starting hybrid server as no router was registered");
            }
        } else {
            warn!("not starting hybrid server as no service was registered");
        }
    }
}
