use std::sync::Arc;

use generic_data::TestEntity;
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_offline::resource::{Project, ResourcePathName, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions, Resource};
use tokio::sync::Mutex;

use crate::{build_manager::BuildManager, DataManager, Transaction};

#[tokio::test]
async fn test_transaction_system() -> anyhow::Result<()> {
    let project_dir = tempfile::tempdir().unwrap();
    let build_dir = project_dir.path().join("temp");
    std::fs::create_dir(&build_dir).unwrap();

    let project = Project::create_new(&project_dir).unwrap();
    let project = Arc::new(Mutex::new(project));

    let mut registry = ResourceRegistryOptions::new();
    registry = generic_data::offline::register_resource_types(registry);
    let registry = registry.create_async_registry();

    let asset_registry = AssetRegistryOptions::new();
    //asset_registry = generic_data_offline::add_loader(asset_registry);
    let asset_registry = asset_registry.create();

    let mut options = DataBuildOptions::new(&build_dir);
    options.content_store(&ContentStoreAddr::from(build_dir.as_path()));
    let build_manager = BuildManager::new(&options, &project_dir, Manifest::default());

    {
        let mut data_manager = DataManager::new(
            project.clone(),
            registry.clone(),
            asset_registry,
            build_manager,
        );
        let resource_path: ResourcePathName = "/entity/create_test.dc".into();

        // Create a new Resource, Edit some properties and Commit it
        let mut transaction = Transaction::new();
        let new_id = transaction.create_resource(resource_path.clone(), TestEntity::TYPE)?;
        transaction.update_property(new_id, "test_string", "\"Update1\"")?;
        transaction.update_property(new_id, "test_bool", "false")?;
        transaction.update_property(new_id, "test_position", "[1,2,3]")?;
        data_manager.commit_transaction(transaction).await?;

        assert!(project.lock().await.exists_named(&resource_path));

        // Delete the created Resource
        let mut transaction = Transaction::new();
        transaction.delete_resource(new_id)?;
        data_manager.commit_transaction(transaction).await?;
        assert!(!project.lock().await.exists_named(&resource_path));
        assert!(!project.lock().await.exists(new_id));

        // Undo delete
        data_manager.undo_transaction().await?;
        assert!(project.lock().await.exists_named(&resource_path));
        assert!(project.lock().await.exists(new_id));

        // Undo Create
        data_manager.undo_transaction().await?;
        assert!(!project.lock().await.exists_named(&resource_path));
        assert!(!project.lock().await.exists(new_id));

        // Redo Create
        data_manager.redo_transaction().await?;
        assert!(project.lock().await.exists_named(&resource_path));
        assert!(project.lock().await.exists(new_id));

        // Redo Delete
        data_manager.redo_transaction().await?;
        assert!(!project.lock().await.exists_named(&resource_path));
        assert!(!project.lock().await.exists(new_id));

        // Create Transaction with invalid edit
        let invalid_resource: ResourcePathName = "/entity/create_invalid.dc".into();
        let mut transaction = Transaction::new();
        let new_id = transaction.create_resource(invalid_resource.clone(), TestEntity::TYPE)?;
        transaction.update_property(new_id, "test_string", "\"Update2\"")?;
        transaction.update_property(new_id, "test_bool", "false")?;
        transaction.update_property(new_id, "INVALID_PROPERTY", "[1,2,3]")?;
        if data_manager.commit_transaction(transaction).await.is_ok() {
            panic!("Transaction with invalid property update shouldn't succceed");
        }
        assert!(!project.lock().await.exists_named(&invalid_resource));

        drop(data_manager);
    }

    drop(registry);
    drop(project);
    Ok(())
}
