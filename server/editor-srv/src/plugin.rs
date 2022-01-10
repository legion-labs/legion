use std::sync::Arc;

use lgn_app::prelude::*;
use lgn_data_transaction::DataManager;
use lgn_ecs::prelude::*;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::setup
                .exclusive_system()
                .after(lgn_resource_registry::ResourceRegistryPluginScheduling::ResourceRegistryCreated)
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        );
    }
}

impl EditorPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        data_manager: Res<'_, Arc<Mutex<DataManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let grpc_server = super::grpc::GRPCServer::new(data_manager.clone());
        grpc_settings.register_service(grpc_server.service());
    }
}
