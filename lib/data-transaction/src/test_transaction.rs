use std::sync::Arc;

use generic_data::offline::TestEntity;
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_reg::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourcePathName, ResourceRegistryOptions};
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistryOptions, Resource, ResourceId, ResourceTypeAndId,
};
use tokio::sync::Mutex;

use crate::{
    build_manager::BuildManager, CloneResourceOperation, CreateResourceOperation, DataManager,
    DeleteResourceOperation, RenameResourceOperation, Transaction, UpdatePropertyOperation,
};

#[allow(clippy::too_many_lines)]
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

    let asset_registry = AssetRegistryOptions::new().create();

    let compilers =
        CompilerRegistryOptions::default().add_compiler(&lgn_compiler_testentity::COMPILER_INFO);

    let options = DataBuildOptions::new(&build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()));

    let build_manager = BuildManager::new(options, &project_dir, Manifest::default()).unwrap();

    {
        let mut data_manager = DataManager::new(
            project.clone(),
            registry.clone(),
            asset_registry,
            build_manager,
        );
        let resource_path: ResourcePathName = "/entity/create_test.dc".into();

        let new_id = ResourceTypeAndId {
            t: TestEntity::TYPE,
            id: ResourceId::new(),
        };

        // Create a new Resource, Edit some properties and Commit it
        let transaction = Transaction::new()
            .add_operation(CreateResourceOperation::new(new_id, resource_path.clone()))
            .add_operation(UpdatePropertyOperation::new(
                new_id,
                "test_string",
                "\"Update1\"",
            ))
            .add_operation(UpdatePropertyOperation::new(new_id, "test_bool", "false"))
            .add_operation(UpdatePropertyOperation::new(
                new_id,
                "test_position",
                "[1,2,3]",
            ));
        data_manager.commit_transaction(transaction).await?;

        assert!(project.lock().await.exists_named(&resource_path));

        // Clone the created Resource
        let clone_name: ResourcePathName = "/entity/test_clone.dc".into();
        let clone_id = ResourceTypeAndId {
            t: TestEntity::TYPE,
            id: ResourceId::new(),
        };
        let transaction = Transaction::new().add_operation(CloneResourceOperation::new(
            new_id,
            clone_id,
            clone_name.clone(),
        ));
        data_manager.commit_transaction(transaction).await?;
        assert!(project.lock().await.exists_named(&clone_name));
        assert!(project.lock().await.exists(clone_id));

        // Rename the clone
        let clone_new_name: ResourcePathName = "/entity/test_clone_rename.dc".into();
        let transaction = Transaction::new().add_operation(RenameResourceOperation::new(
            clone_id,
            clone_new_name.clone(),
        ));
        data_manager.commit_transaction(transaction).await?;
        assert!(project.lock().await.exists_named(&clone_new_name));
        assert!(!project.lock().await.exists_named(&clone_name));

        // Undo Rename
        data_manager.undo_transaction().await?;
        assert!(!project.lock().await.exists_named(&clone_new_name));
        assert!(project.lock().await.exists_named(&clone_name));

        // Undo Clone
        data_manager.undo_transaction().await?;
        assert!(!project.lock().await.exists_named(&clone_name));
        assert!(!project.lock().await.exists(clone_id));

        // Delete the created Resource
        let transaction = Transaction::new().add_operation(DeleteResourceOperation::new(new_id));
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
        let resource_path: ResourcePathName = "/entity/create_test.dc".into();
        let new_id = ResourceTypeAndId {
            t: TestEntity::TYPE,
            id: ResourceId::new(),
        };
        let transaction = Transaction::new()
            .add_operation(CreateResourceOperation::new(new_id, resource_path))
            .add_operation(UpdatePropertyOperation::new(
                new_id,
                "test_string",
                "\"Update2\"",
            ))
            .add_operation(UpdatePropertyOperation::new(new_id, "test_bool", "false"))
            .add_operation(UpdatePropertyOperation::new(
                new_id,
                "INVALID_PROPERTY",
                "[1,2,3]",
            ));
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
