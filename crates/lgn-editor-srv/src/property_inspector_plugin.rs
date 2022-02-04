#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tonic::{codegen::http::status, Request, Response, Status};

use lgn_app::prelude::*;
use lgn_data_model::collector::collect_properties;
use lgn_editor_proto::property_inspector::{
    property_inspector_server::{PropertyInspector, PropertyInspectorServer},
    DeleteArrayElementRequest, DeleteArrayElementResponse, GetResourcePropertiesRequest,
    GetResourcePropertiesResponse, InsertNewArrayElementRequest, InsertNewArrayElementResponse,
    ReorderArrayElementRequest, ReorderArrayElementResponse, ResourceDescription, ResourceProperty,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};

use lgn_data_model::{
    collector::{ItemInfo, PropertyCollector},
    json_utils::{self, get_property_as_json_string},
    TypeDefinition,
};
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_data_transaction::{
    ArrayOperation, DataManager, LockContext, Transaction, UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;

pub(crate) struct PropertyInspectorRPC {
    pub(crate) data_manager: Arc<Mutex<DataManager>>,
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
        data_manager: Res<'_, Arc<Mutex<DataManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let property_inspector = PropertyInspectorServer::new(PropertyInspectorRPC {
            data_manager: data_manager.clone(),
        });
        grpc_settings.register_service(property_inspector);
    }
}

struct ResourcePropertyCollector;
impl PropertyCollector for ResourcePropertyCollector {
    type Item = ResourceProperty;
    fn new_item(item_info: &ItemInfo<'_>) -> anyhow::Result<Self::Item> {
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
                    .map(|enum_variant| Self::Item {
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

        Ok(Self::Item {
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
    async fn get_resource_properties(
        &self,
        request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        let request = request.into_inner();
        let resource_id = parse_resource_id(request.id.as_str())?;

        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let handle = ctx
            .loaded_resource_handles
            .get(resource_id)
            .ok_or_else(|| Status::internal(format!("Invalid ResourceID: {}", resource_id)))?;

        let property_bag = ctx
            .resource_registry
            .get_resource_reflection(resource_id.kind, handle)
            .ok_or_else(|| Status::internal(format!("Invalid ResourceID format: {}", request.id)))
            .map(|reflection| -> anyhow::Result<ResourceProperty> {
                collect_properties::<ResourcePropertyCollector>(reflection)
            })?
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: ResourceTypeAndId::to_string(&resource_id),
                path: ctx
                    .project
                    .resource_name(resource_id.id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string(),
                version: 1,
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

        let mut data_manager = self.data_manager.lock().await;
        {
            let mut transaction = Transaction::new();
            for update in &request.property_updates {
                transaction = transaction.add_operation(UpdatePropertyOperation::new(
                    resource_id,
                    update.name.as_str(),
                    update.json_value.as_str(),
                ));
            }
            data_manager
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

        self.data_manager
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
                request.json_value.as_str(),
            ))
        };

        self.data_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        Ok(Response::new(InsertNewArrayElementResponse {}))
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

        self.data_manager
            .lock()
            .await
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(format!("transaction error {}", err)))?;

        Ok(Response::new(ReorderArrayElementResponse {}))
    }
}
