#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use axum::Router;
use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_data_model::json_utils::get_property_as_json_string;
use lgn_data_offline::resource::{Project, ResourceHandles, ResourcePathName};
use lgn_data_runtime::{
    Resource, ResourceDescriptor, ResourceId, ResourcePathId, ResourceType, ResourceTypeAndId,
};
use lgn_data_transaction::{
    ArrayOperation, CloneResourceOperation, CreateResourceOperation, DeleteResourceOperation,
    LockContext, RenameResourceOperation, ReparentResourceOperation, Transaction,
    TransactionManager, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;

use editor_srv::resource_browser::server::{
    self, CloneResourceRequest, CloneResourceResponse, CloseSceneRequest, CloseSceneResponse,
    CreateResourceRequest, CreateResourceResponse, DeleteResourceRequest, DeleteResourceResponse,
    GetActiveScenesRequest, GetActiveScenesResponse, GetResourceTypeNamesRequest,
    GetResourceTypeNamesResponse, GetRuntimeSceneInfoRequest, GetRuntimeSceneInfoResponse,
    ListAssetsRequest, ListAssetsResponse, OpenSceneRequest, OpenSceneResponse,
    RenameResourceRequest, RenameResourceResponse, ReparentResourceRequest,
    ReparentResourceResponse, SearchResourcesRequest, SearchResourcesResponse,
};
use lgn_online::server::{Error, Result};

use editor_srv::resource_browser::{
    Api, Asset, CloneResource200Response, CreateResource200Response, GetActiveScenes200Response,
    GetResourceTypeNames200Response, GetRuntimeSceneInfo200Response, ListAssets200Response,
    NextSearchToken, ResourceDescription,
};

use lgn_graphics_data::offline_gltf::GltfFile;
use lgn_resource_registry::ResourceRegistrySettings;
use lgn_scene_plugin::SceneMessage;
use lgn_tracing::{error, info, span_scope, warn};
use serde_json::json;
use tokio::sync::Mutex;

pub(crate) struct ResourceBrowserSettings {
    default_scene: String,
}

impl ResourceBrowserSettings {
    pub(crate) fn new(default_scene: String) -> Self {
        Self { default_scene }
    }
}

#[derive(Default)]
pub(crate) struct ResourceBrowserPlugin {
    active_scenes: HashSet<ResourceTypeAndId>,
}

fn parse_resource_id(value: &str) -> Result<ResourceTypeAndId, Error> {
    value
        .parse::<ResourceTypeAndId>()
        .map_err(|_err| Error::bad_request(format!("Invalid ResourceID format: {}", value)))
}

#[derive(Debug)]
struct IndexSnapshot {
    entity_to_parent: HashMap<ResourceTypeAndId, ResourceTypeAndId>,
    parent_to_entities: HashMap<ResourceTypeAndId, Vec<ResourceTypeAndId>>,
    entity_to_names: HashMap<ResourceTypeAndId, (ResourcePathName, ResourcePathName)>,
    name_to_entity: HashMap<ResourcePathName, ResourceTypeAndId>,
}

impl IndexSnapshot {
    async fn new(ctx: &mut LockContext<'_>) -> Self {
        let mut entity_to_parent = HashMap::new();
        let mut parent_to_entities = HashMap::new();
        let mut entity_to_names = HashMap::new();
        let mut name_to_entity = HashMap::new();

        for res_id in ctx.project.resource_list().await {
            if let (Ok(raw_name), Ok(res_name)) = (
                ctx.project.raw_resource_name(res_id).await,
                ctx.project.resource_name(res_id).await,
            ) {
                let mut parent_id = raw_name.extract_parent_info().0;
                if parent_id.is_none() && res_id.kind == sample_data::offline::Entity::TYPE {
                    if let Ok(handle) = ctx.get_or_load(res_id).await {
                        if let Some(entity) =
                            handle.get::<sample_data::offline::Entity>(&ctx.asset_registry)
                        {
                            if let Some(parent) = &entity.parent {
                                parent_id = Some(parent.source_resource()); // Some(parent.resource_id());
                            }
                        }
                    }
                }

                if let Some(parent_id) = parent_id {
                    entity_to_parent.insert(res_id, parent_id);
                    parent_to_entities
                        .entry(parent_id)
                        .or_insert_with(Vec::new)
                        .push(res_id);
                }
                name_to_entity.insert(res_name.clone(), res_id);
                entity_to_names.insert(res_id, (raw_name, res_name));
            }
        }
        Self {
            entity_to_parent,
            parent_to_entities,
            entity_to_names,
            name_to_entity,
        }
    }
}

impl Plugin for ResourceBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::post_setup
                .exclusive_system()
                .after(lgn_resource_registry::ResourceRegistryPluginScheduling::ResourceRegistryCreated)
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer)
        );

        app.add_system(Self::handle_events);
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::load_default_scene
                .exclusive_system()
                .after(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum SelectionOperation {
    Set = 0,
    Add = 1,
    Remove = 2,
}

impl Server {
    pub fn new(world: &mut World) -> Self {
        let (scene_events_tx, scene_events_rx) = crossbeam_channel::unbounded::<SceneMessage>();
        world.insert_resource(scene_events_rx);

        let transaction_manager = world.resource::<Arc<Mutex<TransactionManager>>>();
        let settings = world.resource::<ResourceRegistrySettings>();

        Self {
            transaction_manager: transaction_manager.clone(),
            uploads_folder: settings.root_folder().join("uploads"),
            scene_events_tx,
        }
    }
}

impl ResourceBrowserPlugin {
    fn post_setup(world: &mut World) {
        let server = Arc::new(Server::new(world));

        world
            .resource_mut::<lgn_grpc::SharedRouter>()
            .into_inner()
            .register_routes(server::register_routes, server);
    }

    #[allow(clippy::needless_pass_by_value)]
    fn handle_events(
        scene_events_rx: ResMut<'_, crossbeam_channel::Receiver<SceneMessage>>,
        mut scene_event_writer: EventWriter<'_, '_, SceneMessage>,
    ) {
        for event in scene_events_rx.try_iter() {
            scene_event_writer.send(event);
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn load_default_scene(
        settings: Res<'_, ResourceBrowserSettings>,
        tokio_runtime: ResMut<'_, TokioAsyncRuntime>,
        mut event_writer: EventWriter<'_, '_, SceneMessage>,
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
    ) {
        span_scope!("resource_browser::opening_default_scene");
        if !settings.default_scene.is_empty() {
            lgn_tracing::info!("Opening default scene: {}", settings.default_scene);
            let transaction_manager = transaction_manager.clone();
            tokio_runtime.block_on(async move {
                let mut transaction_manager = transaction_manager.lock().await;

                for scene in settings.default_scene.split_terminator(';') {
                    let resource_path = ResourcePathName::from(scene);

                    let resource_id = LockContext::new(&transaction_manager)
                        .await
                        .project
                        .find_resource(&resource_path)
                        .await;

                    match resource_id {
                        Ok(resource_id) => {
                            // Send OpenScene regardless of the compilation results
                            event_writer.send(SceneMessage::OpenScene(
                                ResourcePathId::from(resource_id)
                                    .push(sample_data::runtime::Entity::TYPE)
                                    .resource_id(),
                            ));

                            match transaction_manager.add_scene(resource_id).await {
                                Ok(_resource_path_id) => {}
                                Err(err) => lgn_tracing::warn!(
                                    "Failed to build scene '{}': {}",
                                    scene,
                                    err.to_string()
                                ),
                            }
                        }
                        Err(error) => {
                            lgn_tracing::warn!(
                                "Failed to locate scene '{}' in project: {}",
                                &resource_path,
                                error
                            );
                        }
                    }
                }
            });
        }
    }
}

// Create a basic entity with Transform Component + Parenting update
fn template_entity(
    name: &str,
    entity_id: ResourceTypeAndId,
    parent_id: Option<ResourceTypeAndId>,
    mut transaction: Transaction,
) -> Transaction {
    transaction = transaction.add_operation(ArrayOperation::insert_element(
        entity_id,
        "components",
        None,
        Some(serde_json::json!({ "Name": { "name" : name} }).to_string()),
    ));

    transaction = transaction.add_operation(ArrayOperation::insert_element(
        entity_id,
        "components",
        None,
        Some(
            serde_json::json!({ "Transform": sample_data::offline::Transform::default() })
                .to_string(),
        ),
    ));

    transaction = update_entity_parenting(entity_id, parent_id, None, transaction, true);

    transaction
}

// Update Parenting info of Entity
fn update_entity_parenting(
    entity_id: ResourceTypeAndId,
    new_parent: Option<ResourceTypeAndId>,
    old_parent: Option<ResourceTypeAndId>,
    mut transaction: Transaction,
    clear_children: bool,
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
                Some(json!(current_path).to_string()),
            ));

            let mut parent_path: ResourcePathId = new_parent.into();
            parent_path = parent_path.push(sample_data::runtime::Entity::TYPE);
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                entity_id,
                &[("parent", json!(parent_path).to_string())],
            ));
        }
    } else {
        // Reset parent property
        transaction = transaction.add_operation(UpdatePropertyOperation::new(
            entity_id,
            &[("parent", "null")],
        ));
    }

    // Reset children (when cloning)
    if clear_children {
        transaction = transaction.add_operation(UpdatePropertyOperation::new(
            entity_id,
            &[("children", serde_json::Value::Array(Vec::new()).to_string())],
        ));
    }

    transaction
}

// Works for both .gltf and .glb (doesn't support external references anymore)
fn create_gltf_resource(gltf_path: &Path) -> Result<PathBuf, Error> {
    let raw_data = std::fs::read(gltf_path).map_err(|err| Error::internal(err.to_string()))?;
    let gltf_file = GltfFile::from_bytes(raw_data);
    let path = gltf_path.with_extension("temp");
    let mut file = std::fs::File::create(&path).map_err(|err| Error::internal(err.to_string()))?;
    gltf_file
        .write(&mut file)
        .map_err(|err| Error::internal(err.to_string()))?;
    Ok(path)
}

pub(crate) struct Server {
    pub(crate) transaction_manager: Arc<Mutex<TransactionManager>>,
    pub(crate) uploads_folder: PathBuf,
    pub(crate) scene_events_tx: crossbeam_channel::Sender<SceneMessage>,
}

#[async_trait]
impl Api for Server {
    /// Search for all resources
    async fn search_resources(
        &self,
        request: SearchResourcesRequest,
    ) -> Result<SearchResourcesResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let resources = ctx.project.resource_list().await;
        let mut descriptors = Vec::new();
        for resource_id in resources {
            let path: String = ctx
                .project
                .resource_name(resource_id)
                .await
                .unwrap_or_else(|_err| "".into())
                .to_string();

            // Basic Filter
            if !request.token.0.is_empty() && !path.contains(&request.token.0) {
                continue;
            }

            descriptors.push(ResourceDescription {
                id: ResourceTypeAndId::to_string(&resource_id),
                path,
                type_: resource_id
                    .kind
                    .as_pretty()
                    .trim_start_matches("offline_")
                    .into(),
                version: 1,
            });
        }

        let next_search_token = NextSearchToken {
            next_search_token: "".to_string(),
            total: u64::try_from(descriptors.len()).unwrap(),
            resource_description: descriptors,
        };

        Ok(SearchResourcesResponse::Status200(next_search_token))
    }

    /// Create a new resource
    async fn create_resource(
        &self,
        request: CreateResourceRequest,
    ) -> Result<CreateResourceResponse> {
        let resource_type = request.body.resource_type.as_str();

        let new_resource_id = ResourceTypeAndId {
            kind: ResourceType::new(
                match resource_type {
                    // gltf resource can be created out of either gltf, glb, and gltfzip"
                    "gltfzip" | "glb" => "gltf",
                    _ => resource_type,
                }
                .as_bytes(),
            ),
            id: ResourceId::new(),
        };

        let name = request.body.resource_name.as_ref().map_or(
            request.body.resource_type.trim_start_matches("offline_"),
            String::as_str,
        );

        let mut parent_id: Option<ResourceTypeAndId> = None;
        let mut resource_path = if let Some(parent_id_str) = &request.body.parent_resource_id {
            parent_id = Some(parse_resource_id(parent_id_str)?);

            let mut res_name = ResourcePathName::new(format!("!{}", parent_id.unwrap()));
            res_name.push(name);
            res_name
        } else {
            ResourcePathName::new(name)
        };

        let mut content_path = request
            .body
            .upload_id
            .as_ref()
            .map(|upload_id| self.uploads_folder.join(upload_id).join(name));

        match resource_type {
            "gltf" | "glb" => {
                if let Some(tmp_content_path) = content_path {
                    content_path =
                        Some(create_gltf_resource(&tmp_content_path).map_err(|err| {
                            Error::internal(format!("failed to create gltf resource: {}", err))
                        })?);
                }
            }
            _ => (),
        };

        let mut transaction = Transaction::new().add_operation(CreateResourceOperation::new(
            new_resource_id,
            resource_path,
            true, // Allow auto-rename
            content_path,
        ));

        // Until we support 'template', Initiate Entity with
        // some TransformComponent and update parenting
        if resource_type == "offline_entity" {
            transaction = template_entity(name, new_resource_id, parent_id, transaction);
        }

        // Add Init Values
        for init_value in request.body.init_values {
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                new_resource_id,
                &[(init_value.property_path, init_value.json_value)],
            ));
        }

        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Error::internal(format!("failed to commit transaction: {}", err)))?;

        // Remove uploads folder after successful upload
        if let Some(upload_id) = &request.body.upload_id {
            std::fs::remove_dir_all(self.uploads_folder.join(upload_id))
                .map_err(|err| Error::internal(format!("failed to remove dir: {}", err)))?;
        }

        Ok(CreateResourceResponse::Status200(
            CreateResource200Response {
                new_id: new_resource_id.to_string(),
            },
        ))
    }

    /// Get the list of all the resources types available (for creation dialog)
    async fn get_resource_type_names(
        &self,
        _request: GetResourceTypeNamesRequest,
    ) -> Result<GetResourceTypeNamesResponse> {
        let mut transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let res_types = ctx.asset_registry.get_resource_types();

        Ok(GetResourceTypeNamesResponse::Status200(
            GetResourceTypeNames200Response {
                resource_types: res_types
                    .into_iter()
                    .map(|(_k, v)| String::from(v))
                    .collect(),
            },
        ))
    }

    /// Delete a Resource
    async fn delete_resource(
        &self,
        request: DeleteResourceRequest,
    ) -> Result<DeleteResourceResponse> {
        let resource_id = parse_resource_id(request.body.id.as_str())?;

        // Build Entity->Parent mapping table. TODO: This should be cached within a index somewhere at one point
        let index_snapshot = {
            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .load_all_resource_type(&[sample_data::offline::Entity::TYPE])
                .await;

            let mut ctx = LockContext::new(&transaction_manager).await;
            IndexSnapshot::new(&mut ctx).await
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

        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Error::internal(format!("delete transaction failed: {}", err)))?;

        Ok(DeleteResourceResponse::Status204)
    }

    /// Rename a Resource
    async fn rename_resource(
        &self,
        request: RenameResourceRequest,
    ) -> Result<RenameResourceResponse> {
        let resource_id = parse_resource_id(request.body.id.as_str())?;

        let mut transaction = Transaction::new().add_operation(RenameResourceOperation::new(
            resource_id,
            ResourcePathName::new(request.body.new_path.as_str()),
        ));
        {
            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Error::internal(format!("rename transaction failed: {}", err)))?;
        }

        Ok(RenameResourceResponse::Status204)
    }

    async fn list_assets(&self, request: ListAssetsRequest) -> Result<ListAssetsResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let mut asset_types = Vec::new();
        for asset_type in &request.body.asset_types {
            asset_types.push(
                ResourceType::from_str(asset_type.as_str())
                    .map_err(|err| Error::internal(format!("list assets failed: {}", err)))?,
            );
        }
        let resources = ctx.project.resource_list().await;
        let mut assets = Vec::new();
        for resource_id in resources {
            if asset_types.contains(&resource_id.kind) {
                let path: String = ctx
                    .project
                    .resource_name(resource_id)
                    .await
                    .unwrap_or_else(|_err| "".into())
                    .to_string();
                assets.push(Asset {
                    id: ResourceTypeAndId::to_string(&resource_id),
                    asset_name: path,
                });
            }
        }

        Ok(ListAssetsResponse::Status200(ListAssets200Response {
            assets,
        }))
    }

    /// Clone a Resource
    async fn clone_resource(&self, request: CloneResourceRequest) -> Result<CloneResourceResponse> {
        let source_resource_id = parse_resource_id(request.body.source_id.as_str())?;

        // Build Entity->Parent mapping table. TODO: This should be cached within a index somewhere at one point
        let index_snapshot = {
            let transaction_manager = self.transaction_manager.lock().await;
            let mut ctx = LockContext::new(&transaction_manager).await;
            IndexSnapshot::new(&mut ctx).await
        };

        // Are we cloning into another target
        let target_parent_id: Option<ResourceTypeAndId> =
            if let Some(target) = &request.body.target_parent_id {
                Some(parse_resource_id(target)?)
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
                    process_queue.extend(
                        children
                            .iter()
                            .copied()
                            .filter(|r| r.kind == sample_data::offline::Entity::TYPE),
                    );
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
                transaction =
                    update_entity_parenting(clone_res_id, parent, None, transaction, true);
            }
        }

        let clone_id = clone_mapping.get(&source_resource_id).unwrap();

        // Add Init Values
        for init_value in request.body.init_values {
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                *clone_id,
                &[(init_value.property_path, init_value.json_value)],
            ));
        }

        let path = {
            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Error::internal(format!("clone transaction failed: {}", err)))?;

            let guard = LockContext::new(&transaction_manager).await;
            guard
                .project
                .resource_name(*clone_id)
                .await
                .map_err(|err| Error::internal(format!("clone transaction failed: {}", err)))?
                .as_str()
                .to_string()
        };

        Ok(CloneResourceResponse::Status200(CloneResource200Response {
            new_resource: ResourceDescription {
                id: clone_id.to_string(),
                path,
                type_: clone_id
                    .kind
                    .as_pretty()
                    .trim_start_matches("offline_")
                    .into(),
                version: 1,
            },
        }))
    }

    /// Reparent a Resource
    async fn reparent_resource(
        &self,
        request: ReparentResourceRequest,
    ) -> Result<ReparentResourceResponse> {
        let resource_id = parse_resource_id(request.body.id.as_str())?;

        let mut new_path = ResourcePathName::new(&request.body.new_path);

        let index_snapshot = {
            let transaction_manager = self.transaction_manager.lock().await;
            let mut ctx = LockContext::new(&transaction_manager).await;
            IndexSnapshot::new(&mut ctx).await
        };
        let new_parent = index_snapshot.name_to_entity.get(&new_path).copied();
        if let Some(new_parent) = new_parent {
            if new_parent == resource_id {
                return Err(Error::bad_request("cannot parent to itself".to_string()));
            }
            new_path = ResourcePathName::new(format!("/!{}", new_parent));
        }

        let old_parent = if resource_id.kind == sample_data::offline::Entity::TYPE {
            index_snapshot.entity_to_parent.get(&resource_id).copied()
        } else {
            None
        };

        // Ignore same reparenting
        if old_parent == new_parent {
            return Ok(ReparentResourceResponse::Status204);
        }

        let mut transaction = Transaction::new().add_operation(ReparentResourceOperation::new(
            resource_id,
            new_path.clone(),
        ));

        transaction =
            update_entity_parenting(resource_id, new_parent, old_parent, transaction, false);

        {
            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Error::internal(format!("reparent transaction failed: {}", err)))?;
        }

        Ok(ReparentResourceResponse::Status204)
    }

    /// Open a Scene
    async fn open_scene(&self, request: OpenSceneRequest) -> Result<OpenSceneResponse> {
        let mut resource_id = parse_resource_id(request.scene_id.0.as_str())?;

        if resource_id.kind != sample_data::offline::Entity::TYPE {
            return Err(Error::internal(format!(
                "Expected Entity in OpenScene. Resource {} is a {}",
                resource_id,
                resource_id.kind.as_pretty()
            )));
        }

        lgn_tracing::info!("Opening scene: {}", resource_id);
        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager
            .add_scene(resource_id)
            .await
            .map_err(|err| Error::internal(err.to_string()))?;

        // Get runtime entity id
        if resource_id.kind == sample_data::offline::Entity::TYPE {
            resource_id = ResourcePathId::from(resource_id)
                .push(sample_data::runtime::Entity::TYPE)
                .resource_id();
        }
        if let Err(err) = self
            .scene_events_tx
            .send(SceneMessage::OpenScene(resource_id))
        {
            warn!("Failed to OpenScene for {}: {}", resource_id, err);
        }

        Ok(OpenSceneResponse::Status204)
    }

    /// Close a Scene
    async fn close_scene(&self, request: CloseSceneRequest) -> Result<CloseSceneResponse> {
        let mut resource_id = parse_resource_id(request.scene_id.0.as_str())?;

        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager.remove_scene(resource_id).await;

        // Get runtime entity id
        if resource_id.kind == sample_data::offline::Entity::TYPE {
            resource_id = ResourcePathId::from(resource_id)
                .push(sample_data::runtime::Entity::TYPE)
                .resource_id();
        }
        lgn_tracing::info!("Closing scene: {:?}", resource_id);
        if let Err(err) = self
            .scene_events_tx
            .send(SceneMessage::CloseScene(resource_id))
        {
            warn!("Failed to Close Scene for {}: {}", resource_id, err);
        }
        Ok(CloseSceneResponse::Status204)
    }

    /// Get active scenes
    async fn get_active_scenes(
        &self,
        _request: GetActiveScenesRequest,
    ) -> Result<GetActiveScenesResponse> {
        let transaction_manager = self.transaction_manager.lock().await;

        Ok(GetActiveScenesResponse::Status200(
            GetActiveScenes200Response {
                scene_ids: transaction_manager
                    .get_active_scenes()
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            },
        ))
    }

    async fn get_runtime_scene_info(
        &self,
        request: GetRuntimeSceneInfoRequest,
    ) -> Result<GetRuntimeSceneInfoResponse> {
        let resource_id = parse_resource_id(request.scene_id.0.as_str())?;

        if resource_id.kind != sample_data::offline::Entity::TYPE {
            return Err(Error::internal(format!(
                "Expected Entity in GetRuntimeSceneInfo. Resource {} is a {}",
                resource_id,
                resource_id.kind.as_pretty()
            )));
        }

        let asset_id = ResourcePathId::from(resource_id)
            .push(sample_data::runtime::Entity::TYPE)
            .resource_id();

        let manifest_id = {
            let transaction_manager = self.transaction_manager.lock().await;

            transaction_manager.get_runtime_manifest_id().await
        };

        lgn_tracing::info!(
            "Playing scene: {}, manifest: {}, root asset: {}",
            resource_id,
            manifest_id,
            asset_id
        );

        Ok(GetRuntimeSceneInfoResponse::Status200(
            GetRuntimeSceneInfo200Response {
                manifest_id: manifest_id.to_string(),
                asset_id: asset_id.to_string(),
            },
        ))
    }
}
