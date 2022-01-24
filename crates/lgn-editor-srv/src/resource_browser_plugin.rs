#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_data_offline::resource::Project;
use lgn_data_offline::{resource::ResourcePathName, ResourcePathId};
use lgn_data_runtime::{Resource, ResourceId, ResourceType, ResourceTypeAndId};
use lgn_data_transaction::{
    ArrayOperation, CloneResourceOperation, CreateResourceOperation, DataManager,
    DeleteResourceOperation, LockContext, RenameResourceOperation, ReparentResourceOperation,
    Transaction, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;
use lgn_editor_proto::property_inspector::UpdateResourcePropertiesRequest;
use lgn_editor_proto::resource_browser::{
    CloneResourceRequest, CloneResourceResponse, DeleteResourceRequest, DeleteResourceResponse,
    GetResourceTypeNamesRequest, GetResourceTypeNamesResponse, ImportResourceRequest,
    ImportResourceResponse, RenameResourceRequest, RenameResourceResponse, ReparentResourceRequest,
    ReparentResourceResponse, SearchResourcesRequest,
};

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tonic::{codegen::http::status, Request, Response, Status};

pub(crate) struct ResourceBrowserRPC {
    pub(crate) data_manager: Arc<Mutex<DataManager>>,
}

#[derive(Default)]
pub(crate) struct ResourceBrowserPlugin {}

fn parse_resource_id(value: &str) -> Result<ResourceTypeAndId, Status> {
    value
        .parse::<ResourceTypeAndId>()
        .map_err(|_err| Status::internal(format!("Invalid ResourceID format: {}", value)))
}

struct IndexSnapshot {
    entity_to_parent: HashMap<ResourceTypeAndId, ResourceTypeAndId>,
    parent_to_entities: HashMap<ResourceTypeAndId, Vec<ResourceTypeAndId>>,
    entity_to_names: HashMap<ResourceTypeAndId, ResourcePathName>,
}

impl IndexSnapshot {
    fn new(project: &Project) -> Self {
        let mut entity_to_parent = HashMap::new();
        let mut parent_to_entities = HashMap::new();
        let mut entity_to_names = HashMap::new();

        for id in project.resource_list() {
            if let Ok(res_name) = project.raw_resource_name(id) {
                let kind = project.resource_info(id).unwrap().0;
                let res_id = ResourceTypeAndId { kind, id };

                if let Some(parent_id) = res_name.extract_parent_info().0 {
                    entity_to_parent.insert(res_id, parent_id);
                    parent_to_entities
                        .entry(parent_id)
                        .or_insert_with(Vec::new)
                        .push(res_id);
                }
                entity_to_names.insert(res_id, res_name);
            }
        }
        Self {
            entity_to_parent,
            parent_to_entities,
            entity_to_names,
        }
    }
}

impl ResourceBrowserPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

use lgn_editor_proto::resource_browser::{
    resource_browser_server::{ResourceBrowser, ResourceBrowserServer},
    CreateResourceRequest, CreateResourceResponse, ResourceDescription, SearchResourcesResponse,
};

impl Plugin for ResourceBrowserPlugin {
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

impl ResourceBrowserPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        tokio_runtime: ResMut<'_, TokioAsyncRuntime>,
        data_manager: Res<'_, Arc<Mutex<DataManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let resource_browser_service = ResourceBrowserServer::new(ResourceBrowserRPC {
            data_manager: data_manager.clone(),
        });
        grpc_settings.register_service(resource_browser_service);

        let default_scene = lgn_config::config_get_or!("editor_srv.default_scene", String::new());
        if !default_scene.is_empty() {
            lgn_tracing::info!("Opening default scene: {}", default_scene);
            let data_manager = data_manager.clone();
            tokio_runtime.start_detached(async move {
                let data_manager = data_manager.lock().await;
                if let Err(err) = data_manager
                    .build_by_name(&default_scene.clone().into())
                    .await
                {
                    lgn_tracing::warn!(
                        "Failed to build default_scene '{}': {}",
                        &default_scene,
                        err.to_string()
                    );
                }
            });
        }
    }
}

// Create a basic entity with Transform Component + Parenting update
fn template_entity(
    entity_id: ResourceTypeAndId,
    parent_id: Option<ResourceTypeAndId>,
    mut transaction: Transaction,
) -> Transaction {
    transaction = transaction.add_operation(ArrayOperation::insert_element(
        entity_id,
        "components",
        None,
        serde_json::json!({ "Transform": sample_data::offline::Transform::default() }).to_string(),
    ));
    transaction = update_entity_parenting(entity_id, parent_id, None, transaction);

    transaction
}

// Update Parenting info of Entity
fn update_entity_parenting(
    entity_id: ResourceTypeAndId,
    new_parent: Option<ResourceTypeAndId>,
    old_parent: Option<ResourceTypeAndId>,
    mut transaction: Transaction,
) -> Transaction {
    let mut current_path: ResourcePathId = entity_id.into();
    current_path = current_path.push(sample_data::runtime::Entity::TYPE);

    // Remove entity from old_parent
    if let Some(old_parent) = old_parent {
        if old_parent.kind == sample_data::offline::Entity::TYPE {
            transaction = transaction.add_operation(ArrayOperation::delete_value(
                old_parent,
                "children",
                json!(current_path).to_string(),
            ));
        }
    }

    // Add entity to new parent and update 'Parent' property
    if let Some(new_parent) = new_parent {
        if new_parent.kind == sample_data::offline::Entity::TYPE {
            transaction = transaction.add_operation(ArrayOperation::insert_element(
                new_parent,
                "children",
                None, // insert at end
                json!(current_path).to_string(),
            ));

            let mut parent_path: ResourcePathId = new_parent.into();
            parent_path = parent_path.push(sample_data::runtime::Entity::TYPE);
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                entity_id,
                "parent",
                json!(parent_path).to_string(),
            ));
        }
    } else {
        // Reset parent property
        transaction =
            transaction.add_operation(UpdatePropertyOperation::new(entity_id, "parent", "null"));
    }

    // Reset children
    transaction = transaction.add_operation(UpdatePropertyOperation::new(
        entity_id,
        "children",
        serde_json::Value::Array(Vec::new()).to_string(),
    ));

    transaction
}

#[tonic::async_trait]
impl ResourceBrowser for ResourceBrowserRPC {
    /// Search for all resources
    async fn search_resources(
        &self,
        request: Request<SearchResourcesRequest>,
    ) -> Result<Response<SearchResourcesResponse>, Status> {
        let request = request.get_ref();
        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let descriptors = ctx
            .project
            .resource_list()
            .filter_map(|resource_id| {
                let path: String = ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string();

                let (kind, _, _) = ctx.project.resource_info(resource_id).unwrap();

                // Basic Filter
                if !request.search_token.is_empty() {
                    path.find(&request.search_token)?;
                }

                Some(ResourceDescription {
                    id: ResourceTypeAndId::to_string(&ResourceTypeAndId {
                        kind,
                        id: resource_id,
                    }),
                    path,
                    version: 1,
                })
            })
            .collect::<Vec<ResourceDescription>>();

        Ok(Response::new(SearchResourcesResponse {
            next_search_token: "".to_string(),
            total: descriptors.len() as u64,
            resource_descriptions: descriptors,
        }))
    }

    /// Create a new resource
    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> Result<Response<CreateResourceResponse>, Status> {
        let request = request.into_inner();

        let new_resource_id = ResourceTypeAndId {
            kind: ResourceType::new(request.resource_type.as_bytes()),
            id: ResourceId::new(),
        };

        let name = request.resource_path.as_ref().map_or(
            request.resource_type.trim_start_matches("offline_"),
            String::as_str,
        );

        let mut parent_id: Option<ResourceTypeAndId> = None;
        let mut resource_path = if let Some(parent_id_str) = &request.parent_resource_id {
            parent_id = Some(parse_resource_id(parent_id_str)?);
            let mut res_name = ResourcePathName::new(format!("!{}", parent_id.unwrap()));
            res_name.push(name);
            res_name
        } else {
            ResourcePathName::new(name)
        };

        let mut transaction = Transaction::new().add_operation(CreateResourceOperation::new(
            new_resource_id,
            resource_path,
            true, // Allow auto-rename
        ));

        // Until we support 'template', Initiate Entity with
        // some TransformComponent and update parenting
        if request.resource_type.as_str() == "offline_entity" {
            transaction = template_entity(new_resource_id, parent_id, transaction);
        }

        // Add Init Values
        for init_value in request.init_values {
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                new_resource_id,
                init_value.property_path,
                init_value.json_value,
            ));
        }

        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CreateResourceResponse {
            new_id: new_resource_id.to_string(),
        }))
    }

    /// Get the list of all the resources types available (for creation dialog)
    async fn get_resource_type_names(
        &self,
        _request: Request<GetResourceTypeNamesRequest>,
    ) -> Result<Response<GetResourceTypeNamesResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let res_types = ctx.resource_registry.get_resource_types();
        Ok(Response::new(GetResourceTypeNamesResponse {
            resource_types: res_types
                .into_iter()
                .map(|(_k, v)| String::from(v))
                .collect(),
        }))
    }

    /// Import a new resource from an existing local file
    async fn import_resource(
        &self,
        _request: Request<ImportResourceRequest>,
    ) -> Result<Response<ImportResourceResponse>, Status> {
        Err(Status::internal(""))
    }

    /// Delete a Resource
    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<DeleteResourceResponse>, Status> {
        let request = request.get_ref();
        let resource_id = parse_resource_id(request.id.as_str())?;

        // Build Entity->Parent mapping table. TODO: This should be cached within a index somewhere at one point
        let index_snapshot = {
            let data_manager = self.data_manager.lock().await;
            let ctx = LockContext::new(&data_manager).await;
            IndexSnapshot::new(&ctx.project)
        };

        // Recursively gather all the children entities as well
        let delete_queue = {
            let mut delete_queue = HashSet::<ResourceTypeAndId>::new();
            // Parse all the resource_id
            let mut process_queue = vec![resource_id];
            while let Some(current_id) = process_queue.pop() {
                if let Some(children) = index_snapshot.parent_to_entities.get(&current_id) {
                    process_queue.extend_from_slice(children);
                }
                delete_queue.insert(current_id);
            }
            delete_queue
        };

        // Create a transaction to delete all the entities
        let mut transaction = Transaction::new();
        for resource_id in &delete_queue {
            transaction = transaction.add_operation(DeleteResourceOperation::new(*resource_id));

            // TEMP: Until with have children discovery, handle the 'children' property of Entity manually
            // If we have a parent_resource that's not getting deleted as well, update its 'children' to remove the current entry
            if resource_id.kind == sample_data::offline::Entity::TYPE {
                if let Some(parent_path_id) = index_snapshot.entity_to_parent.get(resource_id) {
                    if !delete_queue.contains(parent_path_id)
                        && parent_path_id.kind == sample_data::offline::Entity::TYPE
                    {
                        let mut child: ResourcePathId = (*resource_id).into();
                        child = child.push(sample_data::runtime::Entity::TYPE);

                        transaction = transaction.add_operation(ArrayOperation::delete_value(
                            *parent_path_id,
                            "children",
                            json!(child).to_string(),
                        ));
                    }
                }
            }
        }

        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(DeleteResourceResponse {}))
    }

    /// Rename a Resource
    async fn rename_resource(
        &self,
        request: Request<RenameResourceRequest>,
    ) -> Result<Response<RenameResourceResponse>, Status> {
        let request = request.get_ref();
        let resource_id = parse_resource_id(request.id.as_str())?;
        let mut transaction = Transaction::new().add_operation(RenameResourceOperation::new(
            resource_id,
            ResourcePathName::new(request.new_path.as_str()),
        ));
        {
            let mut data_manager = self.data_manager.lock().await;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(RenameResourceResponse {}))
    }

    /// Clone a Resource
    async fn clone_resource(
        &self,
        request: Request<CloneResourceRequest>,
    ) -> Result<Response<CloneResourceResponse>, Status> {
        let request = request.get_ref();
        let source_resource_id = parse_resource_id(request.source_id.as_str())?;

        // Build Entity->Parent mapping table. TODO: This should be cached within a index somewhere at one point
        let index_snapshot = {
            let data_manager = self.data_manager.lock().await;
            let ctx = LockContext::new(&data_manager).await;
            IndexSnapshot::new(&ctx.project)
        };

        // Are we cloning into another target
        let target_parent_id: Option<ResourceTypeAndId> =
            if let Some(target) = &request.target_parent_id {
                Some(parse_resource_id(target.as_str())?)
            } else {
                None
            };

        // Mapping between source_id and clone_id
        let mut clone_mapping = HashMap::<ResourceTypeAndId, ResourceTypeAndId>::new();

        // Remap parent to new target
        if let Some(target_parent_id) = target_parent_id {
            if let Some(current_parent) = index_snapshot.entity_to_parent.get(&source_resource_id) {
                clone_mapping.insert(*current_parent, target_parent_id);
            }
        }

        // Clone the children as well
        // Recursively gather all the children entities as well
        let clone_queue = {
            let mut clone_queue = Vec::<ResourceTypeAndId>::new();
            // Parse all the resource_id
            let mut process_queue = vec![source_resource_id];
            while let Some(current_id) = process_queue.pop() {
                if let Some(children) = index_snapshot.parent_to_entities.get(&current_id) {
                    process_queue.extend_from_slice(children);
                }
                clone_queue.push(current_id);
            }
            clone_queue
        };

        // Create a transaction to clone all the entities
        let mut transaction = Transaction::new();
        for source_resource_id in clone_queue {
            let clone_res_id = ResourceTypeAndId {
                kind: source_resource_id.kind,
                id: ResourceId::new(),
            };
            clone_mapping.insert(source_resource_id, clone_res_id);

            // If we have a parent resource, we might need to remap our parent_resource_id if it's cloned
            let parent = index_snapshot
                .entity_to_parent
                .get(&source_resource_id)
                .map(|parent| clone_mapping.get(parent).copied().unwrap_or(*parent));

            transaction = transaction.add_operation(CloneResourceOperation::new(
                source_resource_id,
                clone_res_id,
                parent,
            ));

            if clone_res_id.kind == sample_data::offline::Entity::TYPE {
                transaction = update_entity_parenting(clone_res_id, parent, None, transaction);
            }
        }

        {
            let mut data_manager = self.data_manager.lock().await;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        };

        Ok(Response::new(CloneResourceResponse {
            new_id: clone_mapping.get(&source_resource_id).unwrap().to_string(),
        }))
    }

    /// Clone a Resource
    async fn reparent_resource(
        &self,
        request: Request<ReparentResourceRequest>,
    ) -> Result<Response<ReparentResourceResponse>, Status> {
        let request = request.get_ref();
        let resource_id = parse_resource_id(request.id.as_str())?;
        let new_parent = parse_resource_id(request.new_parent.as_str())?;

        let mut transaction = Transaction::new()
            .add_operation(ReparentResourceOperation::new(resource_id, new_parent));

        if resource_id.kind == sample_data::offline::Entity::TYPE {
            // Build Entity->Parent mapping table. TODO: This should be cached within a index somewhere at one point
            let index_snapshot = {
                let data_manager = self.data_manager.lock().await;
                let ctx = LockContext::new(&data_manager).await;
                IndexSnapshot::new(&ctx.project)
            };

            let old_parent = index_snapshot.entity_to_parent.get(&resource_id).copied();
            transaction =
                update_entity_parenting(resource_id, Some(new_parent), old_parent, transaction);
        }

        let project = Arc::new(Mutex::new(project));
        {
            let mut data_manager = self.data_manager.lock().await;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(ReparentResourceResponse {}))
    }
}
