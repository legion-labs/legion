use lgn_app::prelude::*;
use lgn_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
use lgn_data_transaction::{
    ArrayOperation, CreateResourceOperation, DataManager, DeleteResourceOperation, LockContext,
    Transaction, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;
use lgn_editor_proto::scene_explorer::{
    scene_explorer_server::{SceneExplorer, SceneExplorerServer},
    CreateEntityRequest, CreateEntityResponse, DeleteEntitiesRequest, DeleteEntitiesResponse,
    EntityInfo, GetEntityHierarchyRequest, GetEntityHierarchyResponse,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

struct SceneExplorerRPC {
    data_manager: Arc<Mutex<DataManager>>,
}

#[derive(Default)]
pub(crate) struct SceneExplorerPlugin {}

impl Plugin for SceneExplorerPlugin {
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

impl SceneExplorerPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        data_manager: Res<'_, Arc<Mutex<DataManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let scene_explorer_service = SceneExplorerServer::new(SceneExplorerRPC {
            data_manager: data_manager.clone(),
        });
        grpc_settings.register_service(scene_explorer_service);
    }
}

fn parse_resource_id(value: impl AsRef<str>) -> Result<ResourceTypeAndId, Status> {
    value
        .as_ref()
        .parse::<ResourceTypeAndId>()
        .map_err(|_err| Status::internal(format!("Invalid ResourceID format: {}", value.as_ref())))
}

fn build_entity_info(
    resource_id: &ResourceTypeAndId,
    project: &Project,
    resource_registry: &ResourceRegistry,
    loaded_resources: &ResourceHandles,
) -> Option<EntityInfo> {
    let path = project
        .resource_name(*resource_id)
        .map_or_else(|_err| String::from("unnamed"), |v| v.to_string());

    if let Some(entity_dc) = loaded_resources
        .get(*resource_id)
        .and_then(|handle| handle.get::<generic_data::offline::EntityDc>(resource_registry))
    {
        return Some(EntityInfo {
            path,
            entity_name: entity_dc.name.clone(),
            r#type: generic_data::offline::EntityDc::TYPENAME.into(),
            resource_id: resource_id.to_string(),
            children: entity_dc
                .children
                .iter()
                .filter_map(|res_path_id| {
                    build_entity_info(
                        &res_path_id.source_resource(),
                        project,
                        resource_registry,
                        loaded_resources,
                    )
                })
                .collect(),
        });
    }
    None
}

#[tonic::async_trait]
impl SceneExplorer for SceneExplorerRPC {
    async fn get_entity_hierarchy(
        &self,
        request: Request<GetEntityHierarchyRequest>,
    ) -> Result<Response<GetEntityHierarchyResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.top_resource_id.as_str())?;

        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let root_entity = build_entity_info(
            &resource_id,
            &ctx.project,
            &ctx.resource_registry,
            &ctx.loaded_resource_handles,
        )
        .ok_or_else(|| Status::internal("failed to build hierarchy"))?;

        Ok(Response::new(GetEntityHierarchyResponse {
            entity_info: Some(root_entity),
        }))
    }

    async fn create_entity(
        &self,
        request: Request<CreateEntityRequest>,
    ) -> Result<Response<CreateEntityResponse>, Status> {
        let request = request.into_inner();

        let mut new_name = String::new();
        let entity_name = request
            .entity_name
            .unwrap_or_else(|| String::from("new_entity"));
        let new_resource_id = ResourceTypeAndId {
            kind: generic_data::offline::EntityDc::TYPE,
            id: ResourceId::new(),
        };

        if let Some(scene_resource_id) = &request.scene_resource_id {
            let scene_resource_id = parse_resource_id(scene_resource_id)?;

            let data_manager = self.data_manager.lock().await;
            let ctx = LockContext::new(&data_manager).await;
            let res_path = ctx
                .project
                .resource_name(scene_resource_id)
                .map_err(|err| Status::internal(err.to_string()))?;

            new_name.push_str(res_path.to_string().as_str());
            new_name.push_str("/_children_/");
            new_name.push_str(new_resource_id.id.to_string().as_str());
        } else {
            new_name.push('/');
            new_name.push_str(&entity_name);
        }

        let mut transaction = Transaction::new().add_operation(CreateResourceOperation::new(
            new_resource_id,
            new_name.into(),
        ));

        transaction = transaction.add_operation(UpdatePropertyOperation::new(
            new_resource_id,
            "name",
            serde_json::Value::String(entity_name).to_string(),
        ));

        if let Some(parent_resource_id) = &request.parent_resource_id {
            let parent_resource_id = parse_resource_id(parent_resource_id)?;
            let mut parent_path: ResourcePathId = parent_resource_id.into();
            parent_path = parent_path.push(generic_data::runtime::EntityDc::TYPE);

            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                new_resource_id,
                "parent",
                serde_json::Value::String(parent_path.to_string()).to_string(),
            ));

            let mut child_path: ResourcePathId = new_resource_id.into();
            child_path = child_path.push(generic_data::runtime::EntityDc::TYPE);
            let value_json = serde_json::Value::String(child_path.to_string()).to_string();

            transaction = transaction.add_operation(ArrayOperation::insert_element(
                parent_resource_id,
                "children",
                None, // insert at end
                value_json.as_str(),
            ));
        }

        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CreateEntityResponse {
            new_id: new_resource_id.to_string(),
        }))
    }

    async fn delete_entities(
        &self,
        request: Request<DeleteEntitiesRequest>,
    ) -> Result<Response<DeleteEntitiesResponse>, Status> {
        let request = request.into_inner();

        // Parse all the resource_id
        let mut process_queue = request
            .resource_ids
            .iter()
            .filter_map(|id| parse_resource_id(id.as_str()).ok())
            .collect::<Vec<ResourceTypeAndId>>();

        // Recursively gather all the children entities as well
        let to_delete = {
            let mut results =
                std::collections::HashMap::<ResourceTypeAndId, Option<ResourceTypeAndId>>::new();
            let data_manager = self.data_manager.lock().await;
            let ctx = LockContext::new(&data_manager).await;

            while let Some(resource_id) = process_queue.pop() {
                if let Some(entity_dc) =
                    ctx.loaded_resource_handles
                        .get(resource_id)
                        .and_then(|handle| {
                            handle.get::<generic_data::offline::EntityDc>(&ctx.resource_registry)
                        })
                {
                    results.insert(
                        resource_id,
                        entity_dc
                            .parent
                            .as_ref()
                            .map(ResourcePathId::source_resource),
                    );
                    // Recursively process children entities
                    for child_res_path_id in &entity_dc.children {
                        process_queue.push(child_res_path_id.source_resource());
                    }
                }
            }
            results
        };

        // Create a transaction to delete all the entities
        let mut transaction = Transaction::new();
        for (resource_id, parent) in &to_delete {
            // Remove the entity from its parent.children list, if the parent is not getting deleted as well.
            if let Some(parent_path_id) = parent {
                if !to_delete.contains_key(parent_path_id) {
                    let mut child: ResourcePathId = (*resource_id).into();
                    child = child.push(generic_data::runtime::EntityDc::TYPE);
                    let child_reference_value = serde_json::to_value(child.to_string())
                        .map_err(|err| Status::internal(err.to_string()))?;

                    transaction = transaction.add_operation(ArrayOperation::delete_value(
                        *parent_path_id,
                        "children",
                        child_reference_value.to_string().as_str(),
                    ));
                }
            }
            transaction = transaction.add_operation(DeleteResourceOperation::new(*resource_id));
        }

        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(DeleteEntitiesResponse {}))
    }
}

#[cfg(test)]
mod test {

    use super::{
        CreateEntityRequest, DataManager, DeleteEntitiesRequest, EntityInfo,
        GetEntityHierarchyRequest, SceneExplorer, SceneExplorerRPC,
    };
    use std::path::Path;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tonic::Request;

    use lgn_content_store::ContentStoreAddr;
    use lgn_data_build::DataBuildOptions;
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
    use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions};

    use lgn_data_transaction::BuildManager;

    fn setup_project(project_dir: impl AsRef<Path>) -> Arc<Mutex<DataManager>> {
        let build_dir = project_dir.as_ref().join("temp");
        std::fs::create_dir_all(&build_dir).unwrap();

        let project = Project::create_new(&project_dir).expect("failed to create a project");
        let mut registry = ResourceRegistryOptions::new();
        generic_data::offline::register_resource_types(&mut registry);
        let registry = registry.create_async_registry();
        let project = Arc::new(Mutex::new(project));

        let asset_registry = AssetRegistryOptions::new().create();
        let compilers =
            CompilerRegistryOptions::default().add_compiler(&lgn_compiler_entitydc::COMPILER_INFO);

        let options = DataBuildOptions::new(&build_dir, compilers)
            .content_store(&ContentStoreAddr::from(build_dir.as_path()));

        let build_manager = BuildManager::new(options, &project_dir, Manifest::default()).unwrap();
        Arc::new(Mutex::new(DataManager::new(
            project,
            registry,
            asset_registry,
            build_manager,
        )))
    }

    fn display_hierarchy(info: &EntityInfo, depth: usize) {
        println!("{:indent$}{}", "", info.entity_name, indent = depth);
        for child_info in &info.children {
            display_hierarchy(child_info, depth + 1);
        }
    }

    #[tokio::test]
    async fn test_scene_explorer() -> anyhow::Result<()> {
        //let project_dir = std::path::PathBuf::from("d:/local_db/");
        //std::fs::remove_dir_all(&project_dir).ok();
        let project_dir = tempfile::tempdir().unwrap();

        {
            let data_manager = setup_project(&project_dir);
            let scene_explorer = SceneExplorerRPC {
                data_manager: data_manager.clone(),
            };

            let top_id = scene_explorer
                .create_entity(Request::new(CreateEntityRequest {
                    entity_name: Some("top".into()),
                    template_id: None,
                    scene_resource_id: None,
                    parent_resource_id: None,
                    init_values: Vec::new(),
                }))
                .await?
                .into_inner()
                .new_id;

            let mut child_ids = Vec::<String>::new();

            for i in 0..2 {
                let child_id = scene_explorer
                    .create_entity(Request::new(CreateEntityRequest {
                        entity_name: Some(format!("child{}", i)),
                        template_id: None,
                        scene_resource_id: Some(top_id.clone()),
                        parent_resource_id: Some(top_id.clone()),
                        init_values: Vec::new(),
                    }))
                    .await?
                    .into_inner()
                    .new_id;
                child_ids.push(child_id.clone());

                for j in 0..2 {
                    let sub_child_id = scene_explorer
                        .create_entity(Request::new(CreateEntityRequest {
                            entity_name: Some(format!("subchild{}", j)),
                            template_id: None,
                            scene_resource_id: Some(top_id.clone()),
                            parent_resource_id: Some(child_id.clone()),
                            init_values: Vec::new(),
                        }))
                        .await?
                        .into_inner()
                        .new_id;
                    child_ids.push(sub_child_id);
                }
            }

            {
                let response = scene_explorer
                    .get_entity_hierarchy(Request::new(GetEntityHierarchyRequest {
                        filter: String::new(),
                        top_resource_id: top_id.clone(),
                    }))
                    .await?
                    .into_inner();

                display_hierarchy(&response.entity_info.unwrap(), 0);
            }

            // Try delete a child
            scene_explorer
                .delete_entities(Request::new(DeleteEntitiesRequest {
                    resource_ids: vec![child_ids[0].clone()],
                }))
                .await?;

            // Try Undo
            {
                let mut guard = data_manager.lock().await;
                guard.undo_transaction().await?;
            }
        }
        Ok(())
    }
}
