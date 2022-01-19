//! `gRPC` plugin for Legion's ECS.
//!
//! Provides `gRPC` server support to the engine, compatible with the `tonic`
//! crate.

// crate-specific lint exceptions:
//#![allow()]

use std::net::SocketAddr;

use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_online::grpc::{
    multiplexer_service::{MultiplexableService, MultiplexerService, MultiplexerServiceBuilder},
    Server,
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
    multiplexer_service_builder: MultiplexerServiceBuilder,
}

impl GRPCPluginSettings {
    pub fn new(grpc_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr,
            multiplexer_service_builder: MultiplexerService::builder(),
        }
    }

    pub fn register_service<S>(&mut self, s: S) -> &mut Self
    where
        S: MultiplexableService + NamedService + Send + Sync + 'static,
    {
        self.multiplexer_service_builder.add_service(s);

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
pub struct GRPCPlugin;

impl Plugin for GRPCPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GRPCPluginSettings>()
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                Self::start_grpc_server
                    .exclusive_system()
                    .label(GRPCPluginScheduling::StartRpcServer),
            );
    }
}

impl GRPCPlugin {
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
}
