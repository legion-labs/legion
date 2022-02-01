use tonic::{Request, Status};

use lgn_editor_proto::property_inspector::property_inspector_server::PropertyInspector;

use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
use lgn_editor_proto::property_inspector::GetResourcePropertiesRequest;

use lgn_data_transaction::{CreateResourceOperation, Transaction};

#[tokio::test]
async fn test_property_inspector() -> anyhow::Result<()> {
    let project_dir = tempfile::tempdir().unwrap();

    {
        let data_manager = crate::test_resource_browser::setup_project(&project_dir).await;
        let property_inspector = crate::property_inspector_plugin::PropertyInspectorRPC {
            data_manager: data_manager.clone(),
        };

        // Create a dummy Scene Entity

        let new_id = {
            let new_id = ResourceTypeAndId {
                kind: sample_data::offline::Entity::TYPE,
                id: ResourceId::new(),
            };

            let transaction = Transaction::new().add_operation(CreateResourceOperation::new(
                new_id,
                ResourcePathName::new("dummy_entity"),
                false,
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
            assert_eq!(desc.path.as_str(), "/dummy_entity");
            assert_eq!(desc.id, new_id.to_string());
            assert_eq!(response.properties[0].ptype, "Entity");
            assert_eq!(response.properties[0].sub_properties[0].name, "children");
        }
    }
    Ok(())
}
