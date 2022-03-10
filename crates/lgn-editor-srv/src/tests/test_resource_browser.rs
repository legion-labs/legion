use std::{path::Path, str::FromStr, sync::Arc};

use serde_json::json;
use tokio::sync::Mutex;
use tonic::Request;

use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions, Resource, ResourceTypeAndId};
use lgn_data_transaction::{
    ArrayOperation, BuildManager, SelectionManager, Transaction, TransactionManager,
};
use lgn_editor_proto::resource_browser::{
    resource_browser_server::ResourceBrowser, CloneResourceRequest, CreateResourceRequest,
    DeleteResourceRequest, GetResourceTypeNamesRequest, InitPropertyValue, RenameResourceRequest,
};
use lgn_math::Vec3;

/*fn add_scripting_component(root_entity_id: &ResourceTypeAndId) -> Transaction {
    let script_id = ResourceTypeAndId {
        kind: <lgn_scripting::offline::Script as Resource>::TYPE,
        id: ResourceId::new(),
    };

    Transaction::new()
            .add_operation(CreateResourceOperation::new(
                script_id,
                ResourcePathName::new("root_script"),
                false,
                None,
            ))
            .add_operation(UpdatePropertyOperation::new(
                script_id,
                "script",
                json!(
                    r#"
                            pub fn entry() {
                                //print("Hello world!");
                            }

                            pub fn arg() -> i64 {
                                10
                            }

                            pub fn fibonacci(n: i64) -> i64 {
                                if n <= 1 {
                                    n
                                } else {
                                    fibonacci(n - 1) + fibonacci(n - 2)
                                }
                            }"#
                )
                .to_string(),
            )).add_operation(ArrayOperation::insert_element(
                *root_entity_id,
                    "components",
                    None,
                    json!({ "ScriptComponent": lgn_scripting::offline::ScriptComponent {
                        input_values: Vec::new(),
                        entry_fn: String::default(),
                        script_id: Some(ResourcePathId::from(script_id).push(lgn_scripting::runtime::Script::TYPE)),
                        temp_script: String::default(),

                    }})
                    .to_string(),
                ))
}*/

pub(crate) async fn setup_project(project_dir: impl AsRef<Path>) -> Arc<Mutex<TransactionManager>> {
    let build_dir = project_dir.as_ref().join("temp");
    std::fs::create_dir_all(&build_dir).unwrap();

    let project = Project::create_with_remote_mock(&project_dir)
        .await
        .expect("failed to create a project");

    let mut resource_registry = ResourceRegistryOptions::new();
    sample_data::offline::register_resource_types(&mut resource_registry);
    lgn_scripting::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    sample_data::offline::add_loaders(&mut asset_registry);
    lgn_scripting::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create();

    let compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO);

    let options = DataBuildOptions::new(&build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()))
        .asset_registry(asset_registry.clone());

    let build_manager =
        BuildManager::new(options, &project, Manifest::default(), Manifest::default())
            .await
            .unwrap();
    let project = Arc::new(Mutex::new(project));

    Arc::new(Mutex::new(TransactionManager::new(
        project,
        resource_registry,
        asset_registry,
        build_manager,
        SelectionManager::create(),
    )))
}

#[tokio::test]
async fn test_resource_browser() -> anyhow::Result<()> {
    //let project_dir = std::path::PathBuf::from("d:/local_db/");
    //std::fs::remove_dir_all(&project_dir.join("offline")).ok();
    //std::fs::remove_dir_all(&project_dir.join("remote")).ok();
    let project_dir = tempfile::tempdir().unwrap();

    {
        let transaction_manager = setup_project(&project_dir).await;
        let resource_browser = crate::resource_browser_plugin::ResourceBrowserRPC {
            transaction_manager: transaction_manager.clone(),
            uploads_folder: "".into(),
        };

        // Read all Resource Type registered
        let response = resource_browser
            .get_resource_type_names(Request::new(GetResourceTypeNamesRequest {}))
            .await?
            .into_inner();

        // Validate that sceneEntity should be in the list
        assert!(
            response
                .resource_types
                .iter()
                .filter(|res_type| res_type.as_str() == sample_data::offline::Entity::TYPENAME)
                .count()
                == 1
        );

        // Create new resource
        let root_entity_id = resource_browser
            .create_resource(Request::new(CreateResourceRequest {
                resource_type: sample_data::offline::Entity::TYPENAME.into(),
                resource_name: Some("root_entity_".into()),
                parent_resource_id: None,
                init_values: vec![InitPropertyValue {
                    property_path: "components[Transform].position".into(),
                    json_value: json!(Vec3::ZERO).to_string(),
                }],
            }))
            .await?
            .into_inner()
            .new_id;

        let root_entity_id = ResourceTypeAndId::from_str(&root_entity_id).unwrap();
        // Rename the created resource
        resource_browser
            .rename_resource(Request::new(RenameResourceRequest {
                id: root_entity_id.to_string(),
                new_path: "root_entity".into(),
            }))
            .await?;

        // Add Script + ScriptComponent
        /*{
            let transaction = add_scripting_component(&root_entity_id);
            let mut guard = transaction_manager.lock().await;
            guard.commit_transaction(transaction).await.unwrap();
        }*/

        // Create an Hierarchy of Child->SubChild with increment_name
        {
            let offsets: Vec<f32> = vec![-1.0, 0.0, 1.0];

            let colors: Vec<lgn_graphics_data::Color> = vec![
                (255, 0, 0).into(),
                (0, 255, 0).into(),
                (0, 0, 255).into(),
                (255, 255, 0).into(),
                (0, 255, 255).into(),
                (255, 0, 255).into(),
            ];
            let mut color_id = 0;

            #[allow(clippy::needless_range_loop)]
            for i in 0..3u16 {
                let child_id = resource_browser
                    .create_resource(Request::new(CreateResourceRequest {
                        resource_type: sample_data::offline::Entity::TYPENAME.into(),
                        resource_name: Some("child".into()),
                        parent_resource_id: Some(root_entity_id.to_string()),
                        init_values: vec![InitPropertyValue {
                            property_path: "components[Transform].position".into(),
                            json_value: json!(Vec3::new(offsets[i as usize], 0.0, 0.0,))
                                .to_string(),
                        }],
                    }))
                    .await
                    .unwrap()
                    .into_inner()
                    .new_id;

                // Test Renaming the child
                if i == 0 {
                    resource_browser
                        .rename_resource(Request::new(RenameResourceRequest {
                            id: child_id.clone(),
                            new_path: "renamed_child".into(),
                        }))
                        .await?;
                }

                let sub_child_id = resource_browser
                    .create_resource(Request::new(CreateResourceRequest {
                        resource_type: sample_data::offline::Entity::TYPENAME.into(),
                        resource_name: Some("subchild".into()),
                        parent_resource_id: Some(child_id.clone()),
                        init_values: Vec::new(),
                    }))
                    .await
                    .unwrap()
                    .into_inner()
                    .new_id;

                let sub_child_id = ResourceTypeAndId::from_str(&sub_child_id).unwrap();
                let transaction = Transaction::new()
                    .add_operation(ArrayOperation::insert_element(
                        sub_child_id,
                        "components",
                        None,
                        Some(
                            serde_json::json!({
                                "Light" : {}
                            })
                            .to_string(),
                        ),
                    ))
                    .add_operation(ArrayOperation::insert_element(
                        sub_child_id,
                        "components",
                        None,
                        Some(
                            serde_json::json!({
                                "Visual" : {}
                            })
                            .to_string(),
                        ),
                    ));

                let mut guard = transaction_manager.lock().await;
                guard.commit_transaction(transaction).await.unwrap();

                color_id = (color_id + 1) % colors.len();
            }
        }

        // Clone Hierarchy
        let clone_id = resource_browser
            .clone_resource(Request::new(CloneResourceRequest {
                source_id: root_entity_id.to_string(),
                target_parent_id: None, // Same Parent
                init_values: vec![InitPropertyValue {
                    property_path: "components[Transform].position".into(),
                    json_value: json!(Vec3::new(0.0, 0.0, 2.0,)).to_string(),
                }],
            }))
            .await?
            .into_inner()
            .new_resource
            .unwrap()
            .id;

        // Delete Clone
        resource_browser
            .delete_resource(Request::new(DeleteResourceRequest { id: clone_id }))
            .await?;

        {
            let mut guard = transaction_manager.lock().await;
            guard.undo_transaction().await?; // Undo delete
            guard.undo_transaction().await?; // Undo clone
        }
    }
    Ok(())
}
