use legion_app::prelude::*;
use legion_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use std::sync::{Arc, Mutex};

pub struct EditorPlugin {}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let project = app
            .world
            .get_resource::<Arc<Mutex<Project>>>()
            .expect("the editor plugin requires Project resource");

        let registry = app
            .world
            .get_resource::<Arc<Mutex<ResourceRegistry>>>()
            .expect("the editor plugin requires ResourceRegistry resource");

        let resource_handles = app
            .world
            .get_resource::<Arc<Mutex<ResourceHandles>>>()
            .expect("the editor plugin requires ResourceHandles resource");

        let grpc_server = super::grpc::GRPCServer::new(
            project.clone(),
            registry.clone(),
            resource_handles.clone(),
        );

        app.world
            .get_resource_mut::<legion_grpc::GRPCPluginSettings>()
            .expect("the editor plugin requires the gRPC plugin")
            .into_inner()
            .register_service(grpc_server.service());
    }
}
