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
    HybridServer, RestServer, Server,
};
use lgn_tracing::warn;
use tonic::transport::NamedService;

/// Label to use for scheduling systems for GRPC Service registration
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum GRPCPluginScheduling {
    StartRpcServer,
}

pub struct GRPCPluginSettings {
    pub grpc_server_addr: Option<SocketAddr>,
    pub rest_server_addr: Option<SocketAddr>,
    multiplexer_service_builder: Option<MultiplexerServiceBuilder>,
}

impl GRPCPluginSettings {
    pub fn new(grpc_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr: Some(grpc_server_addr),
            rest_server_addr: None,
            multiplexer_service_builder: Some(MultiplexerService::builder()),
        }
    }

    pub fn hybrid(grpc_server_addr: SocketAddr, rest_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr: Some(grpc_server_addr),
            rest_server_addr: Some(rest_server_addr),
            multiplexer_service_builder: Some(MultiplexerService::builder()),
        }
    }

    pub fn rest(rest_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr: None,
            rest_server_addr: Some(rest_server_addr),
            multiplexer_service_builder: None,
        }
    }

    pub fn register_service<S>(&mut self, s: S) -> &mut Self
    where
        S: MultiplexableService + NamedService + Send + Sync + 'static,
    {
        self.multiplexer_service_builder
            .as_mut()
            .unwrap()
            .add_service(s);

        self
    }
}

impl Default for GRPCPluginSettings {
    fn default() -> Self {
        Self::new("[::1]:50051".parse().unwrap())
    }
}

pub struct SharedRouter(Arc<Mutex<Router>>);

impl SharedRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_routes<F, A>(&mut self, register_routes: F, api: A)
    where
        F: FnOnce(Router, A) -> Router,
        A: Clone + Send + Sync + 'static,
    {
        let mut router = self.0.lock().unwrap();

        *router = register_routes(router.clone(), api);
    }
}

impl Default for SharedRouter {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Router::new())))
    }
}

#[derive(PartialEq)]
pub enum GRPCPluginType {
    GRPC,
    Rest,
    Hybrid,
}

// Provides gRPC server capabilities to the engine.
pub struct GRPCPlugin {
    plugin_type: GRPCPluginType,
}

impl Default for GRPCPlugin {
    fn default() -> Self {
        Self {
            plugin_type: GRPCPluginType::GRPC,
        }
    }
}

impl Plugin for GRPCPlugin {
    fn build(&self, app: &mut App) {
        if self.plugin_type == GRPCPluginType::Hybrid || self.plugin_type == GRPCPluginType::Rest {
            app.init_resource::<SharedRouter>();
        }

        let system = match self.plugin_type {
            GRPCPluginType::GRPC => Self::start_grpc_server
                .exclusive_system()
                .label(GRPCPluginScheduling::StartRpcServer),
            GRPCPluginType::Hybrid => Self::start_hybrid_server
                .exclusive_system()
                .label(GRPCPluginScheduling::StartRpcServer),
            GRPCPluginType::Rest => Self::start_rest_server
                .exclusive_system()
                .label(GRPCPluginScheduling::StartRpcServer),
        };

        app.init_resource::<GRPCPluginSettings>()
            .add_startup_system_to_stage(StartupStage::PostStartup, system);
    }
}

impl GRPCPlugin {
    pub fn hybrid() -> Self {
        Self {
            plugin_type: GRPCPluginType::Hybrid,
        }
    }
    pub fn rest_only() -> Self {
        Self {
            plugin_type: GRPCPluginType::Rest,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn start_grpc_server(
        settings: Res<'_, GRPCPluginSettings>,
        rt: ResMut<'_, lgn_async::TokioAsyncRuntime>,
    ) {
        if let Some(service) = settings
            .multiplexer_service_builder
            .as_ref()
            .unwrap()
            .build()
        {
            let server = Server::default().set_listen_address(settings.grpc_server_addr.unwrap());

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
        router: Res<'_, SharedRouter>,
    ) {
        if let Some(service) = settings
            .multiplexer_service_builder
            .as_ref()
            .unwrap()
            .build()
        {
            let mut server =
                HybridServer::default().set_grpc_listen_address(settings.grpc_server_addr.unwrap());

            if let Some(rest_server_addr) = settings.rest_server_addr {
                server = server.set_rest_listen_address(rest_server_addr);
            }

            let router = Arc::clone(&router.0);

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
            warn!("not starting hybrid server as no service was registered");
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn start_rest_server(
        settings: Res<'_, GRPCPluginSettings>,
        rt: ResMut<'_, lgn_async::TokioAsyncRuntime>,
        router: Res<'_, SharedRouter>,
    ) {
        let rest_server_addr = settings.rest_server_addr.unwrap();
        let server = RestServer::default().set_rest_listen_address(rest_server_addr);

        let router = Arc::clone(&router.0);

        rt.start_detached(async move {
            match server.run(router).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("Rest server stopped and no longer listening ({})", e);

                    Err(e)
                }
            }
        });
    }
}
