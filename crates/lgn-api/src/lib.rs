//! `Api` plugin for Legion's ECS.
//!
//! Provides Api server support to the engine.

// crate-specific lint exceptions:
//#![allow()]

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::Router;
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_online::api::Server;
use lgn_tracing::warn;

/// Label to use for scheduling systems for Api Service registration
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum ApiPluginScheduling {
    StartServer,
}

pub struct ApiPluginSettings {
    pub server_addr: SocketAddr,
}

impl ApiPluginSettings {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
}

impl Default for ApiPluginSettings {
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

// Provides rest server capabilities to the engine.
#[derive(Debug, Default)]
pub struct ApiPlugin;

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedRouter>();

        let system = Self::start_server
            .exclusive_system()
            .label(ApiPluginScheduling::StartServer);

        app.init_resource::<ApiPluginSettings>()
            .add_startup_system_to_stage(StartupStage::PostStartup, system);
    }
}

impl ApiPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::needless_pass_by_value)]
    fn start_server(
        settings: Res<'_, ApiPluginSettings>,
        rt: ResMut<'_, lgn_async::TokioAsyncRuntime>,
        router: Res<'_, SharedRouter>,
    ) {
        let server = Server::default().set_rest_listen_address(settings.server_addr);

        let router = Arc::clone(&router.0);

        rt.start_detached(async move {
            let result = server.run(router).await;

            if let Err(ref err) = result {
                warn!("Rest server stopped and no longer listening ({})", err);
            }

            result
        });
    }
}
