#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::sync::Arc;
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

struct PropertyInspectorRPC {
    data_manager: Arc<Mutex<DataManager>>,
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
        let json_value = if let TypeDefinition::Primitive(primitive_descriptor) = item_info.type_def
        {
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
            Some(String::from_utf8(output).unwrap())
        } else {
            None
        };

        Ok(Self::Item {
            name: item_info
                .field_descriptor
                .map_or(String::new(), |field| field.field_name.clone())
                + item_info.suffix.unwrap_or_default(),
            ptype: item_info.type_def.get_type_name().into(),
            json_value,
            sub_properties: Vec::new(),
            attributes: item_info
                .field_descriptor
                .map_or(std::collections::HashMap::default(), |field| {
                    field.attributes.clone()
                }),
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
                    .resource_name(resource_id)
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
                request.index as usize,
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

#[cfg(test)]
mod test {

    use super::{DataManager, PropertyInspector, PropertyInspectorRPC};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    use lgn_editor_proto::resource_browser::{CreateResourceRequest, CreateResourceResponse};

    use generic_data::offline::TestEntity;
    use lgn_content_store::ContentStoreAddr;
    use lgn_data_build::DataBuildOptions;
    use lgn_data_compiler::compiler_reg::CompilerRegistryOptions;
    use lgn_data_offline::resource::{
        Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
    };
    use lgn_data_runtime::{
        manifest::Manifest, AssetRegistryOptions, Resource, ResourceId, ResourceType,
        ResourceTypeAndId,
    };
    use lgn_editor_proto::property_inspector::{
        GetResourcePropertiesRequest, ResourcePropertyUpdate, UpdateResourcePropertiesRequest,
    };

    use tempfile::TempDir;

    use lgn_data_transaction::{BuildManager, CreateResourceOperation, Transaction};

    #[tokio::test]
    async fn test_property_inspector() -> anyhow::Result<()> {
        let project_dir = tempfile::tempdir().unwrap();
        let build_dir = project_dir.path().join("temp");
        std::fs::create_dir(&build_dir).unwrap();

        let project = Project::create_new(&project_dir).expect("failed to create a project");
        // register Scene Type
        let mut registry = ResourceRegistryOptions::new();
        generic_data::offline::register_resource_types(&mut registry);
        let registry = registry.create_async_registry();
        let project = Arc::new(Mutex::new(project));

        let asset_registry = AssetRegistryOptions::new().create();
        let compilers = CompilerRegistryOptions::default()
            .add_compiler(&lgn_compiler_testentity::COMPILER_INFO);

        let options = DataBuildOptions::new(&build_dir, compilers)
            .content_store(&ContentStoreAddr::from(build_dir.as_path()));

        let build_manager = BuildManager::new(options, &project_dir, Manifest::default()).unwrap();

        {
            let data_manager = Arc::new(Mutex::new(DataManager::new(
                project.clone(),
                registry.clone(),
                asset_registry,
                build_manager,
            )));
            let property_inspector = PropertyInspectorRPC {
                data_manager: data_manager.clone(),
            };

            // Create a dummy Scene Entity

            let new_id = {
                let new_id = ResourceTypeAndId {
                    kind: TestEntity::TYPE,
                    id: ResourceId::new(),
                };

                let transaction = Transaction::new().add_operation(CreateResourceOperation::new(
                    new_id,
                    ResourcePathName::new("test/root_entity.ent"),
                ));

                let mut data_manager = data_manager.lock().await;
                data_manager
                    .commit_transaction(transaction)
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;

                new_id
            };

            // Get properties for the newly create Resource
            {
                let response = property_inspector
                    .get_resource_properties(Request::new(GetResourcePropertiesRequest {
                        id: new_id.to_string(),
                    }))
                    .await?
                    .into_inner();

                let desc = response.description.unwrap();
                assert_eq!(desc.path.as_str(), "/test/root_entity.ent");
                assert_eq!(desc.id, new_id.to_string());
                assert_eq!(response.properties[0].ptype, "TestEntity");
                assert_eq!(response.properties[0].sub_properties[0].name, "test_string");
            }

            // Change 'name' property and validate
            {
                property_inspector
                    .update_resource_properties(Request::new(UpdateResourcePropertiesRequest {
                        id: new_id.to_string(),
                        version: 1,
                        property_updates: vec![ResourcePropertyUpdate {
                            name: "test_string".into(),
                            json_value: "\"TestString\"".into(),
                        }],
                    }))
                    .await?;

                let response = property_inspector
                    .get_resource_properties(Request::new(GetResourcePropertiesRequest {
                        id: new_id.to_string(),
                    }))
                    .await?
                    .into_inner();

                assert_eq!(
                    response.properties[0].sub_properties[0]
                        .json_value
                        .as_ref()
                        .unwrap(),
                    "\"TestString\""
                );
            }
        }
        Ok(())
    }
}
