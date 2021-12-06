use std::sync::Arc;

use lgn_app::prelude::*;
use lgn_data_transaction::DataManager;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let data_manager = app
            .world
            .get_resource::<Arc<Mutex<DataManager>>>()
            .expect("the editor plugin requires Project resource");

        let grpc_server = super::grpc::GRPCServer::new(data_manager.clone());

        app.world
            .get_resource_mut::<lgn_grpc::GRPCPluginSettings>()
            .expect("the editor plugin requires the gRPC plugin")
            .into_inner()
            .register_service(grpc_server.service());
    }
}
