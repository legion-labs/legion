use std::net::SocketAddr;

use legion_app::prelude::*;

struct EditorPluginSettings {
    grpc_server_addr: SocketAddr,
}

impl Default for EditorPluginSettings {
    fn default() -> Self {
        Self {
            grpc_server_addr: "[::1]:50052".parse().unwrap(),
        }
    }
}

pub struct EditorPlugin {}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world
            .remove_resource::<EditorPluginSettings>()
            .map_or_else(EditorPluginSettings::default, |x| x);
        let grpc_server = super::grpc::GRPCServer::new();

        // Let's limit our usage of the Async runtime, as this keeps a mutable
        // reference on the world.
        {
            let async_rt = app
                .world
                .get_resource_mut::<legion_async::TokioAsyncRuntime>()
                .expect("the editor plugin requires the async plugin")
                .into_inner();

            async_rt.start_detached(grpc_server.listen_and_serve(settings.grpc_server_addr));
        }
    }
}
