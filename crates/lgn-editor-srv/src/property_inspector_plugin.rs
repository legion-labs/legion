#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use lgn_scene_plugin::SceneMessage;
use lgn_tracing::error;
use sample_data::offline::GltfLoader;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};
use tonic::{codegen::http::status, Request, Response, Status};

use lgn_app::prelude::*;

use async_trait::async_trait;
use editor_srv::{
    common::ResourceDescription,
    property_inspector::{
        server::{
            register_routes, DeletePropertiesArrayItemRequest, DeletePropertiesArrayItemResponse,
            GetAvailableDynTraitsRequest, GetAvailableDynTraitsResponse, GetPropertiesRequest,
            GetPropertiesResponse, InsertPropertyArrayItemRequest, InsertPropertyArrayItemResponse,
            ReorderPropertyArrayRequest, ReorderPropertyArrayResponse, UpdatePropertiesRequest,
            UpdatePropertiesResponse, UpdatePropertySelectionRequest,
            UpdatePropertySelectionResponse,
        },
        Api, InsertPropertyArrayItem200Response, ResourceDescriptionProperties, ResourceProperty,
    },
};
use lgn_api::SharedRouter;
use lgn_app::prelude::*;
use lgn_data_model::{
    collector::{collect_properties, ItemInfo, PropertyCollector},
    json_utils::{self, get_property_as_json_string},
    utils::find_property,
    ReflectionError, TypeDefinition,
};
use lgn_data_runtime::{
    AssetRegistry, Resource, ResourceDescriptor, ResourceId, ResourcePathId, ResourceType,
    ResourceTypeAndId,
};
use lgn_data_transaction::{
    ArrayOperation, LockContext, Transaction, TransactionManager, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;
use lgn_online::server::{Error, Result};

use crate::editor::EditorEvent;

pub(crate) struct PropertyInspectorRPC {
    pub(crate) transaction_manager: Arc<Mutex<TransactionManager>>,
    pub(crate) event_sender: broadcast::Sender<EditorEvent>,
}

#[derive(Default)]
pub(crate) struct PropertyInspectorPlugin;

impl PropertyInspectorPlugin {
    pub fn new() -> Self {
        Self
    }
}

fn parse_resource_id(value: &str) -> Result<ResourceTypeAndId, Status> {
    value
        .parse::<ResourceTypeAndId>()
        .map_err(|_err| Status::internal(format!("Invalid ResourceID format: {}", value)))
}

impl Plugin for PropertyInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::setup
                .exclusive_system()
                .after(lgn_resource_registry::ResourceRegistryPluginScheduling::ResourceRegistryCreated)
                .before(lgn_api::ApiPluginScheduling::StartServer),
        );
    }
}

impl PropertyInspectorPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        mut router: ResMut<'_, SharedRouter>,
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        event_sender: Res<'_, broadcast::Sender<EditorEvent>>,
    ) {
        let server = Arc::new(Server::new(
            transaction_manager.clone(),
            event_sender.clone(),
        ));

        router.register_routes(register_routes, server);
    }
}

struct ResourcePropertyCollector;

impl PropertyCollector for ResourcePropertyCollector {
    type Item = ResourceProperty;

    fn new_item(item_info: &ItemInfo<'_>) -> Result<Self::Item, ReflectionError> {
        let mut name = item_info
            .field_descriptor
            .map_or(String::new(), |field| field.field_name.clone())
            + item_info.suffix.unwrap_or_default();

        let mut ptype: String = item_info.type_def.get_type_name().into();
        let mut sub_properties = Vec::new();
        let mut attributes = None;
        if let Some(field_desc) = &item_info.field_descriptor {
            attributes = field_desc
                .attributes
                .as_ref()
                .map(|attrs| attrs.clone().into_iter().collect());
        }

        let mut json_value: Option<String> = None;

        match item_info.type_def {
            TypeDefinition::Struct(struct_descriptor) => {
                attributes = struct_descriptor
                    .attributes
                    .as_ref()
                    .map(|attrs| attrs.clone().into_iter().collect());
            }

            TypeDefinition::Primitive(primitive_descriptor) => {
                let mut output = Vec::new();
                let mut json = serde_json::Serializer::new(&mut output);

                let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                #[allow(unsafe_code)]
                unsafe {
                    (primitive_descriptor.base_descriptor.dynamic_serialize)(
                        item_info.base,
                        &mut serializer,
                    )?;
                }
                json_value = Some(String::from_utf8(output).unwrap());
            }
            TypeDefinition::Enum(enum_descriptor) => {
                let mut output = Vec::new();
                let mut json = serde_json::Serializer::new(&mut output);

                let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                #[allow(unsafe_code)]
                unsafe {
                    (enum_descriptor.base_descriptor.dynamic_serialize)(
                        item_info.base,
                        &mut serializer,
                    )?;
                }
                json_value = Some(String::from_utf8(output).unwrap());
                ptype = format!("_enum_:{}", ptype);

                sub_properties = enum_descriptor
                    .variants
                    .iter()
                    .map(|enum_variant| {
                        Box::new(ResourceProperty {
                            name: enum_variant.variant_name.clone(),
                            ptype: "_enumvariant_".into(),
                            json_value: Some(
                                serde_json::json!(enum_variant.variant_name).to_string(),
                            ),
                            sub_properties: Vec::new(),
                            attributes: enum_variant
                                .attributes
                                .as_ref()
                                .map_or(std::collections::BTreeMap::default(), |attrs| {
                                    attrs.clone().into_iter().collect()
                                }),
                        })
                    })
                    .collect();
            }
            _ => {}
        }

        Ok(ResourceProperty {
            name,
            ptype,
            json_value,
            sub_properties,
            attributes: attributes.unwrap_or_default(),
        })
    }

    fn add_child(parent: &mut Self::Item, child: Self::Item) {
        let sub_properties = &mut parent.sub_properties;

        // If there's a 'Group' attribute, find or create a PropertyBag for the Group within the parent
        if let Some(group_name) = child.attributes.get("group") {
            // Search for the Group within the Parent SubProperties

            let group_bag = if let Some(group_bag) = sub_properties
                .iter_mut()
                .find(|bag| bag.ptype == "_group_" && bag.name == *group_name)
            {
                group_bag
            } else {
                // Create a new group bag if not found
                sub_properties.push(Box::new(Self::Item {
                    name: group_name.into(),
                    ptype: "_group_".into(),
                    json_value: None,
                    attributes: BTreeMap::new(),
                    sub_properties: Vec::new(),
                }));
                sub_properties.last_mut().unwrap()
            };

            // Add child to group
            group_bag.sub_properties.push(Box::new(child));
        } else {
            sub_properties.push(Box::new(child));
        }
    }
}

pub(crate) struct Server {
    pub(crate) transaction_manager: Arc<Mutex<TransactionManager>>,
    pub(crate) event_sender: broadcast::Sender<EditorEvent>,
}

impl Server {
    fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        event_sender: broadcast::Sender<EditorEvent>,
    ) -> Self {
        Self {
            transaction_manager,
            event_sender,
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn get_properties(&self, request: GetPropertiesRequest) -> Result<GetPropertiesResponse> {
        let resource_id = parse_resource_id(&request.resource_id.0)
            .map_err(|_err| Error::bad_request("invalid resource id"))?;

        let resource = {
            let mut transaction_manager = self.transaction_manager.lock().await;
            let ctx = LockContext::new(&transaction_manager).await;
            ctx.project
                .load_resource_untyped(resource_id)
                .await
                .map_err(|err| Error::internal(err.to_string()))?
        };

        let mut property_bag =
            collect_properties::<ResourcePropertyCollector>(resource.as_reflect())
                .map_err(|err| Error::internal(err.to_string()))?;

        // Add Id property
        property_bag.sub_properties.insert(
            0,
            Box::new(ResourceProperty {
                name: "id".into(),
                ptype: "String".into(),
                sub_properties: Vec::new(),
                json_value: Some(serde_json::json!(resource_id.id.to_string()).to_string()),
                attributes: {
                    let mut attr = BTreeMap::new();
                    attr.insert("readonly".into(), "true".into());
                    attr
                },
            }),
        );

        let path = {
            let transaction_manager = self.transaction_manager.lock().await;
            let ctx = LockContext::new(&transaction_manager).await;
            ctx.project
                .resource_name(resource_id)
                .await
                .unwrap_or_else(|_err| "".into())
                .to_string()
        };

        Ok(GetPropertiesResponse::Status200(
            ResourceDescriptionProperties {
                description: ResourceDescription {
                    id: editor_srv::common::ResourceId(ResourceTypeAndId::to_string(&resource_id)),
                    path,
                    version: 1,
                    type_: resource_id
                        .kind
                        .as_pretty()
                        .trim_start_matches("offline_")
                        .into(),
                },
                properties: vec![property_bag],
            },
        ))
    }

    async fn update_properties(
        &self,
        request: UpdatePropertiesRequest,
    ) -> Result<UpdatePropertiesResponse> {
        let resource_id = parse_resource_id(request.resource_id.0.as_str())
            .map_err(|_err| Error::bad_request("invalid resource id"))?;

        {
            let mut transaction = Transaction::new();

            let mut updates = request.body.updates.clone();

            // HACK!!!
            // Pre-fill GlftLoader component
            /*if let Some(mut property) = updates
                .iter_mut()
                .find(|update| update.name == "components.[Visual].renderable_geometry")
            {
                let result = {
                    ResourcePathId::from_str(
                        property
                            .json_value
                            .as_str()
                            .trim_start_matches('"')
                            .trim_end_matches('"'),
                    )
                    .ok()
                };

                if let Some(res_path_id) = result {
                    let gltf_resource_id = res_path_id.source_resource();
                    if gltf_resource_id.kind == GltfFile::TYPE {
                        let asset_registry = self.asset_registry.clone();
                        let mut gltf_loader = GltfLoader::default();

                        if let Ok(gltf_handle) = asset_registry
                            .load_async::<GltfFile>(gltf_resource_id)
                            .await
                        {
                            let gltf = gltf_handle.get().unwrap();
                            let models = gltf.gather_models(gltf_resource_id);
                            let materials = gltf.gather_materials(gltf_resource_id);

                            for (model, name) in &models {
                                gltf_loader.models.push(
                                    ResourcePathId::from(gltf_resource_id)
                                        .push_named(lgn_graphics_data::offline::Model::TYPE, name)
                                        .push(lgn_graphics_data::runtime::Model::TYPE),
                                );
                                gltf_loader
                                    .materials
                                    .extend(model.meshes.iter().filter_map(|m| m.material.clone()));
                            }

                            for (material, _) in &materials {
                                if let Some(t) = &material.albedo {
                                    gltf_loader.textures.push(t.clone());
                                }
                                if let Some(t) = &material.normal {
                                    gltf_loader.textures.push(t.clone());
                                }
                                if let Some(t) = &material.roughness {
                                    gltf_loader.textures.push(t.clone());
                                }
                                if let Some(t) = &material.metalness {
                                    gltf_loader.textures.push(t.clone());
                                }
                            }
                        }

                        // Fix up the edit if the model is invalid
                        if !gltf_loader.models.is_empty()
                            && !gltf_loader.models.contains(&res_path_id)
                        {
                            property.json_value =
                                serde_json::json!(gltf_loader.models.first().unwrap()).to_string();
                        }

                        if let Ok(entity_handle) = asset_registry.load_async(resource_id).await {
                            if let Some(mut entity) =
                                asset_registry.edit::<sample_data::offline::Entity>(&entity_handle)
                            {
                                entity
                                    .components
                                    .retain(|component| !component.is::<GltfLoader>());
                                entity.components.push(Box::new(gltf_loader));

                                asset_registry.commit(entity);
                            }
                        }
                        if let Err(_err) = self
                            .event_sender
                            .send(EditorEvent::ResourceChanged(vec![resource_id]))
                        {
                        }
                    }
                }
            }*/

            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                resource_id,
                &updates
                    .iter()
                    .map(|update| (&update.name, &update.json_value))
                    .collect::<Vec<_>>(),
            ));

            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Error::internal(format!("transaction error {}", err)))?;
        }

        Ok(UpdatePropertiesResponse::Status204)
    }

    async fn get_available_dyn_traits(
        &self,
        request: GetAvailableDynTraitsRequest,
    ) -> Result<GetAvailableDynTraitsResponse> {
        let available_traits = match request.trait_name.as_str() {
            "dyn Component" => {
                let mut results = vec![];
                for entry in inventory::iter::<lgn_data_runtime::ComponentFactory> {
                    results.push(entry.name.into());
                }
                results.sort();
                Some(results)
            }
            _ => None,
        };

        if let Some(available_traits) = available_traits {
            Ok(GetAvailableDynTraitsResponse::Status200(available_traits))
        } else {
            Err(Error::internal(format!(
                "Unknown factory '{}'",
                request.trait_name
            )))
        }
    }

    async fn insert_property_array_item(
        &self,
        request: InsertPropertyArrayItemRequest,
    ) -> Result<InsertPropertyArrayItemResponse> {
        let resource_id = parse_resource_id(request.resource_id.0.as_str())
            .map_err(|_err| Error::bad_request("invalid resource id"))?;

        let transaction = {
            // Remove indices in reverse order to maintain indices
            Transaction::new().add_operation(ArrayOperation::insert_element(
                resource_id,
                request.body.array_path.as_str(),
                Some(request.body.index as usize),
                request.body.json_value,
            ))
        };

        self.transaction_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Error::internal(format!("transaction error {}", err)))?;

        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        let edit = ctx
            .edit_resource(resource_id)
            .await
            .map_err(|err| Error::internal(err.to_string()))?;

        //let mut indexed_path = format!("{}[{}]", request.array_path, request.index);
        let array_prop = find_property(edit.as_reflect(), &request.body.array_path)
            .map_err(|err| Error::internal(format!("transaction error {}", err)))?;

        if let TypeDefinition::Array(array_desc) = array_prop.type_def {
            let mut base = array_prop.base;
            let mut type_def = array_desc.inner_type;
            base = (array_desc.get)(base, request.body.index as usize)
                .map_err(|err| Error::internal(format!("transaction error {}", err)))?;

            let array_subscript = if let TypeDefinition::BoxDyn(box_desc) = array_desc.inner_type {
                type_def = (box_desc.get_inner_type)(base);
                base = (box_desc.get_inner)(base);
                format!("[{}]", type_def.get_type_name())
            } else {
                format!("[{}]", request.body.index)
            };

            let resource_property = ItemInfo {
                base,
                field_descriptor: None,
                type_def,
                suffix: Some(&array_subscript),
                depth: 0,
            }
            .collect::<ResourcePropertyCollector>()
            .map_err(|err| Error::internal(format!("transaction error {}", err)))?
            .unwrap();

            Ok(InsertPropertyArrayItemResponse::Status200(
                InsertPropertyArrayItem200Response {
                    new_value: Some(resource_property),
                },
            ))
        } else {
            Err(Error::internal("Invalid Array Descriptor"))
        }
    }

    async fn delete_properties_array_item(
        &self,
        mut request: DeletePropertiesArrayItemRequest,
    ) -> Result<DeletePropertiesArrayItemResponse> {
        let resource_id = parse_resource_id(request.resource_id.0.as_str())
            .map_err(|_err| Error::bad_request("invalid resource id"))?;

        let transaction = {
            // Remove indices in reverse order to maintain indices
            request.body.indices.sort_unstable();
            let mut transaction = Transaction::new();
            for index in request.body.indices.iter().rev() {
                transaction = transaction.add_operation(ArrayOperation::delete_element(
                    resource_id,
                    request.body.array_path.as_str(),
                    *index as usize,
                ));
            }
            transaction
        };

        self.transaction_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Error::internal(format!("transaction error {}", err)))?;

        Ok(DeletePropertiesArrayItemResponse::Status204)
    }

    async fn reorder_property_array(
        &self,
        request: ReorderPropertyArrayRequest,
    ) -> Result<ReorderPropertyArrayResponse> {
        let resource_id = parse_resource_id(request.resource_id.0.as_str())
            .map_err(|_err| Error::bad_request("invalid resource id"))?;

        let transaction = {
            Transaction::new().add_operation(ArrayOperation::reorder_element(
                resource_id,
                request.body.array_path.as_str(),
                request.body.old_index as usize,
                request.body.new_index as usize,
            ))
        };

        self.transaction_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Error::internal(format!("transaction error {}", err)))?;

        Ok(ReorderPropertyArrayResponse::Status204)
    }

    async fn update_property_selection(
        &self,
        request: UpdatePropertySelectionRequest,
    ) -> Result<UpdatePropertySelectionResponse> {
        let resource_id = request
            .resource_id
            .0
            .parse::<ResourceTypeAndId>()
            .map_err(|_err| {
                Error::bad_request(format!(
                    "Invalid ResourceID format: {}",
                    request.resource_id.0
                ))
            })?;

        let transaction = Transaction::new().add_operation(
            lgn_data_transaction::SelectionOperation::set_selection(&[resource_id]),
        );

        {
            let mut transaction_manager = self.transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Error::internal(err.to_string()))?;
        };

        Ok(UpdatePropertySelectionResponse::Status204)
    }
}
