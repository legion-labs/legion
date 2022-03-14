#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tonic::{codegen::http::status, Request, Response, Status};

use lgn_app::prelude::*;
use lgn_editor_proto::property_inspector::{
    property_inspector_server::{PropertyInspector, PropertyInspectorServer},
    DeleteArrayElementRequest, DeleteArrayElementResponse, GetAvailableDynTraitsRequest,
    GetAvailableDynTraitsResponse, GetResourcePropertiesRequest, GetResourcePropertiesResponse,
    InsertNewArrayElementRequest, InsertNewArrayElementResponse, ReorderArrayElementRequest,
    ReorderArrayElementResponse, ResourceDescription, ResourceProperty, ResourcePropertyUpdate,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse, UpdateSelectionRequest,
    UpdateSelectionResponse,
};

use lgn_data_model::{
    collector::{collect_properties, ItemInfo, PropertyCollector},
    json_utils::{self, get_property_as_json_string},
    utils::find_property,
    ReflectionError, TypeDefinition,
};
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_data_transaction::{
    ArrayOperation, LockContext, Transaction, TransactionManager, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;

pub(crate) struct PropertyInspectorRPC {
    pub(crate) transaction_manager: Arc<Mutex<TransactionManager>>,
}

#[derive(Default)]
pub(crate) struct PropertyInspectorPlugin {}

impl PropertyInspectorPlugin {
    pub fn new() -> Self {
        Self {}
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
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        );
    }
}

impl PropertyInspectorPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let property_inspector = PropertyInspectorServer::new(PropertyInspectorRPC {
            transaction_manager: transaction_manager.clone(),
        });
        grpc_settings.register_service(property_inspector);
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
            attributes = field_desc.attributes.as_ref().cloned();
        }

        let mut json_value: Option<String> = None;

        match item_info.type_def {
            TypeDefinition::Struct(struct_descriptor) => {
                attributes = struct_descriptor.attributes.as_ref().cloned();
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
                    .map(|enum_variant| ResourceProperty {
                        name: enum_variant.variant_name.clone(),
                        ptype: "_enumvariant_".into(),
                        json_value: Some(serde_json::json!(enum_variant.variant_name).to_string()),
                        sub_properties: Vec::new(),
                        attributes: enum_variant.attributes.as_ref().map_or(
                            std::collections::HashMap::default(),
                            std::clone::Clone::clone,
                        ),
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
                sub_properties.push(Self::Item {
                    name: group_name.into(),
                    ptype: "_group_".into(),
                    ..ResourceProperty::default()
                });
                sub_properties.last_mut().unwrap()
            };

            // Add child to group
            group_bag.sub_properties.push(child);
        } else {
            sub_properties.push(child);
        }
    }
}

#[tonic::async_trait]
impl PropertyInspector for PropertyInspectorRPC {
    async fn update_selection(
        &self,
        request: Request<UpdateSelectionRequest>,
    ) -> Result<Response<UpdateSelectionResponse>, Status> {
        let request = request.into_inner();

        let resource_id = request
            .resource_id
            .parse::<ResourceTypeAndId>()
            .map_err(|_err| {
                Status::internal(format!(
                    "Invalid ResourceID format: {}",
                    request.resource_id
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
                .map_err(|err| Status::internal(err.to_string()))?;
        };
        Ok(Response::new(UpdateSelectionResponse {}))
    }

    async fn get_resource_properties(
        &self,
        request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.id.as_str())?;

        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        let handle = ctx
            .get_or_load(resource_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let mut property_bag = if let Some(reflection) = ctx
            .resource_registry
            .get_resource_reflection(resource_id.kind, &handle)
        {
            collect_properties::<ResourcePropertyCollector>(reflection)
                .map_err(|err| Status::internal(err.to_string()))?
        } else {
            // Return a default bag if there's no reflection
            ResourceProperty {
                name: "".into(),
                ptype: resource_id.kind.as_pretty().into(),
                json_value: None,
                attributes: HashMap::new(),
                sub_properties: Vec::new(),
            }
        };

        // Add Id property
        property_bag.sub_properties.insert(
            0,
            ResourceProperty {
                name: "id".into(),
                ptype: "String".into(),
                sub_properties: Vec::new(),
                json_value: Some(serde_json::json!(resource_id.id.to_string()).to_string()),
                attributes: {
                    let mut attr = HashMap::new();
                    attr.insert("readonly".into(), "true".into());
                    attr
                },
            },
        );

        Ok(Response::new(GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: ResourceTypeAndId::to_string(&resource_id),
                path: ctx
                    .project
                    .resource_name(resource_id.id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string(),
                version: 1,
                r#type: resource_id
                    .kind
                    .as_pretty()
                    .trim_start_matches("offline_")
                    .into(),
            }),
            properties: vec![property_bag],
        }))
    }

    async fn update_resource_properties(
        &self,
        request: Request<UpdateResourcePropertiesRequest>,
    ) -> Result<Response<UpdateResourcePropertiesResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.id.as_str())?;

        let mut transaction_manager = self.transaction_manager.lock().await;
        {
            let mut transaction = Transaction::new();
            transaction = transaction.add_operation(UpdatePropertyOperation::new(
                resource_id,
                &request
                    .property_updates
                    .iter()
                    .map(|update| (&update.name, &update.json_value))
                    .collect::<Vec<_>>(),
            ));
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(format!("transaction error {}", err)))?;
        }
        Ok(Response::new(UpdateResourcePropertiesResponse {}))
    }

    async fn delete_array_element(
        &self,
        request: Request<DeleteArrayElementRequest>,
    ) -> Result<Response<DeleteArrayElementResponse>, Status> {
        let mut request = request.into_inner();
        let resource_id = parse_resource_id(request.resource_id.as_str())?;
        let transaction = {
            // Remove indices in reverse order to maintain indices
            request.indices.sort_unstable();
            let mut transaction = Transaction::new();
            for index in request.indices.iter().rev() {
                transaction = transaction.add_operation(ArrayOperation::delete_element(
                    resource_id,
                    request.array_path.as_str(),
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
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        Ok(Response::new(DeleteArrayElementResponse {}))
    }

    async fn insert_new_array_element(
        &self,
        request: Request<InsertNewArrayElementRequest>,
    ) -> Result<Response<InsertNewArrayElementResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.resource_id.as_str())?;
        let transaction = {
            // Remove indices in reverse order to maintain indices
            Transaction::new().add_operation(ArrayOperation::insert_element(
                resource_id,
                request.array_path.as_str(),
                Some(request.index as usize),
                request.json_value,
            ))
        };

        self.transaction_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        let handle = ctx
            .get_or_load(resource_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection(resource_id.kind, &handle)
            .ok_or_else(|| {
                Status::internal(format!("Invalid ResourceID format: {}", resource_id))
            })?;

        //let mut indexed_path = format!("{}[{}]", request.array_path, request.index);
        let array_prop = find_property(reflection, &request.array_path)
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        if let TypeDefinition::Array(array_desc) = array_prop.type_def {
            let mut base = array_prop.base;
            let mut type_def = array_desc.inner_type;
            base = (array_desc.get)(base, request.index as usize)
                .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

            let array_subscript = if let TypeDefinition::BoxDyn(box_desc) = array_desc.inner_type {
                type_def = (box_desc.get_inner_type)(base);
                base = (box_desc.get_inner)(base);
                format!("[{}]", type_def.get_type_name())
            } else {
                format!("[{}]", request.index)
            };

            let resource_property = ItemInfo {
                base,
                field_descriptor: None,
                type_def,
                suffix: Some(&array_subscript),
                depth: 0,
            }
            .collect::<ResourcePropertyCollector>()
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

            Ok(Response::new(InsertNewArrayElementResponse {
                new_value: Some(resource_property),
            }))
        } else {
            Err(Status::internal("Invalid Array Descriptor"))
        }
    }

    async fn reorder_array_element(
        &self,
        request: Request<ReorderArrayElementRequest>,
    ) -> Result<Response<ReorderArrayElementResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.resource_id.as_str())?;
        let transaction = {
            Transaction::new().add_operation(ArrayOperation::reorder_element(
                resource_id,
                request.array_path.as_str(),
                request.old_index as usize,
                request.new_index as usize,
            ))
        };

        self.transaction_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        Ok(Response::new(ReorderArrayElementResponse {}))
    }

    async fn get_available_dyn_traits(
        &self,
        request: Request<GetAvailableDynTraitsRequest>,
    ) -> Result<Response<GetAvailableDynTraitsResponse>, Status> {
        let request = request.get_ref();

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
            Ok(Response::new(GetAvailableDynTraitsResponse {
                available_traits,
            }))
        } else {
            Err(Status::internal(format!(
                "Unknown factory '{}'",
                request.trait_name
            )))
        }
    }
}
