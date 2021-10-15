use legion_app::prelude::*;

pub struct EditorPlugin {}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let grpc_server = super::grpc::GRPCServer::new();

        app.world
            .get_resource_mut::<legion_grpc::GRPCPluginSettings>()
            .expect("the editor plugin requires the gRPC plugin")
            .into_inner()
            .register_service(grpc_server.service());
    }
}
