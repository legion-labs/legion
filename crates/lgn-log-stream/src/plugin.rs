use lgn_app::prelude::{App, Plugin, StartupStage};
use lgn_ecs::prelude::{ExclusiveSystemDescriptorCoercion, IntoExclusiveSystem, Res, ResMut};
use lgn_grpc::GRPCPluginSettings;

use crate::grpc::{GRPCServer, TraceEventsReceiver};

#[derive(Default)]
pub struct LogStreamPlugin {}

impl Plugin for LogStreamPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::setup
                .exclusive_system()
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        );
    }
}

impl LogStreamPlugin {
    fn setup(
        receiver: Res<'_, TraceEventsReceiver>,
        mut grpc_settings: ResMut<'_, GRPCPluginSettings>,
    ) {
        let grpc_server = GRPCServer::new(receiver.clone());

        grpc_settings.register_service(grpc_server.service());

        drop(receiver);
    }
}
