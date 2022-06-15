use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{ResourceDescriptor, ResourceTypeAndId};
use lgn_data_transaction::{CreateResourceOperation, Transaction};
use lgn_editor_proto::property_inspector::property_inspector_server::PropertyInspector;
use lgn_editor_yaml::property_inspector::{
    server::{GetPropertiesRequest, GetPropertiesResponse},
    Api, ResourceDescriptionProperties, ResourceId,
};
use lgn_governance::api::{space::SpaceId, workspace::WorkspaceId};
use tokio::sync::broadcast;
use tonic::{Request, Status};

#[tokio::test]
async fn test_grpc_property_inspector() -> anyhow::Result<()> {
    let project_dir = tempfile::tempdir().unwrap();
    let (editor_events_sender, _editor_events_receiver) = broadcast::channel(1_000);

    {
        let transaction_manager = crate::test_resource_browser::setup_project(&project_dir).await;

        let property_inspector = crate::property_inspector_plugin::PropertyInspectorRPC {
            transaction_manager: transaction_manager.clone(),
            event_sender: editor_events_sender.clone(),
        };

        // Create a dummy Scene Entity

        let new_id = {
            let new_id = ResourceTypeAndId {
                kind: sample_data::offline::Entity::TYPE,
                id: lgn_data_runtime::ResourceId::new(),
            };

            let transaction = Transaction::new().add_operation(CreateResourceOperation::new(
                new_id,
                ResourcePathName::new("dummy_entity"),
                false,
                None,
            ));

            let mut transaction_manager = transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;

            new_id
        };

        // Try to create all the register Components
        {
            let response = property_inspector
                .get_available_dyn_traits(Request::new(
                    lgn_editor_proto::property_inspector::GetAvailableDynTraitsRequest {
                        trait_name: "dyn Component".into(),
                    },
                ))
                .await?
                .into_inner();

            print!("creating {} components: ", response.available_traits.len());
            for component_type in response.available_traits {
                print!("{}, ", component_type);
                property_inspector
                    .insert_new_array_element(Request::new(
                        lgn_editor_proto::property_inspector::InsertNewArrayElementRequest {
                            resource_id: new_id.to_string(),
                            array_path: "components".into(),
                            index: 0,
                            json_value: Some(
                                serde_json::json!({
                            component_type : {} })
                                .to_string(),
                            ),
                        },
                    ))
                    .await?;
            }
        }

        // Get properties for the newly create Resource
        {
            let response = property_inspector
                .get_resource_properties(Request::new(
                    lgn_editor_proto::property_inspector::GetResourcePropertiesRequest {
                        id: new_id.to_string(),
                    },
                ))
                .await?
                .into_inner();

            let desc = response.description.unwrap();
            assert_eq!(desc.path.as_str(), "/dummy_entity");
            assert_eq!(desc.id, new_id.to_string());
            assert_eq!(response.properties[0].ptype, "Entity");
            assert_eq!(response.properties[0].sub_properties[0].name, "id");
            assert_eq!(response.properties[0].sub_properties[1].name, "children");
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_property_inspector() -> anyhow::Result<()> {
    let project_dir = tempfile::tempdir().unwrap();
    let (editor_events_sender, _editor_events_receiver) = broadcast::channel(1_000);

    {
        let transaction_manager = crate::test_resource_browser::setup_project(&project_dir).await;

        let property_inspector = crate::property_inspector_plugin::Server {
            transaction_manager: transaction_manager.clone(),
            event_sender: editor_events_sender.clone(),
        };

        // Create a dummy Scene Entity

        let new_id = {
            let new_id = ResourceTypeAndId {
                kind: sample_data::offline::Entity::TYPE,
                id: lgn_data_runtime::ResourceId::new(),
            };

            let transaction = Transaction::new().add_operation(CreateResourceOperation::new(
                new_id,
                ResourcePathName::new("dummy_entity"),
                false,
                None,
            ));

            let mut transaction_manager = transaction_manager.lock().await;
            transaction_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;

            new_id
        };

        // TODO: Fix when the method is implemented
        // Try to create all the register Components
        // {
        //     let response = property_inspector
        //         .get_available_dyn_traits(Request::new(GetAvailableDynTraitsRequest {
        //             trait_name: "dyn Component".into(),
        //         }))
        //         .await?
        //         .into_inner();

        //     print!("creating {} components: ", response.available_traits.len());
        //     for component_type in response.available_traits {
        //         print!("{}, ", component_type);
        //         property_inspector
        //             .insert_new_array_element(Request::new(InsertNewArrayElementRequest {
        //                 resource_id: new_id.to_string(),
        //                 array_path: "components".into(),
        //                 index: 0,
        //                 json_value: Some(
        //                     serde_json::json!({
        //                     component_type : {} })
        //                     .to_string(),
        //                 ),
        //             }))
        //             .await?;
        //     }
        // }

        // Get properties for the newly create Resource
        {
            let (parts, _) = http::Request::new("").into_parts();

            let response = property_inspector
                .get_properties(GetPropertiesRequest {
                    space_id: SpaceId("0".to_string()),
                    workspace_id: WorkspaceId("0".to_string()),
                    resource_id: ResourceId(new_id.to_string()),
                    parts,
                })
                .await?;

            if let GetPropertiesResponse::Status200(ResourceDescriptionProperties {
                description,
                properties,
            }) = response
            {
                assert_eq!(description.path.as_str(), "/dummy_entity");
                assert_eq!(description.id, new_id.to_string());
                assert_eq!(properties[0].ptype, "Entity");
                assert_eq!(properties[0].sub_properties[0].name, "id");
                assert_eq!(properties[0].sub_properties[1].name, "children");
            } else {
                return Err(anyhow::anyhow!("invalid response received: {:?}", response));
            }
        }
    }

    Ok(())
}
