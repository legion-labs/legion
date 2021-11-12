use generic_data_offline::TestEntity;
use legion_data_offline::resource::{Project, ResourcePathName, ResourceRegistryOptions};

use crate::{DataManager, Transaction};
use legion_data_runtime::{Resource, ResourceId};
use std::sync::{Arc, Mutex};

#[test]
fn test_transaction_system() -> anyhow::Result<()> {
    let project_dir = tempfile::tempdir().unwrap();
    let project_dir = project_dir.path().join("temp");
    std::fs::create_dir(&project_dir).unwrap();

    let project = Project::create_new(project_dir).unwrap();
    let project = Arc::new(Mutex::new(project));

    let mut registry = ResourceRegistryOptions::new();
    registry = generic_data_offline::register_resource_types(registry);
    let registry = registry.create_registry();

    {
        let mut data_manager = DataManager::new(project.clone(), registry.clone());
        let resource_path: ResourcePathName = "/entity/create_test.dc".into();

        // Create a new Resource, Edit some properties and Commit it
        let mut create_entity = || -> anyhow::Result<ResourceId> {
            let mut transaction = Transaction::new();
            let new_id = transaction.create_resource(resource_path.clone(), TestEntity::TYPE)?;
            transaction.update_property(new_id, "test_string", "\"Update1\"")?;
            transaction.update_property(new_id, "test_bool", "false")?;
            transaction.update_property(new_id, "test_position", "[1,2,3]")?;
            data_manager.commit_transaction(transaction)?;
            Ok(new_id)
        };
        let new_id = create_entity()?;
        assert!(project.lock().unwrap().exists_named(&resource_path));

        // Delete the created Resource
        let mut delete_entity = || -> anyhow::Result<()> {
            let mut transaction = Transaction::new();
            transaction.delete_resource(new_id)?;
            data_manager.commit_transaction(transaction)?;
            Ok(())
        };
        delete_entity()?;
        assert!(!project.lock().unwrap().exists_named(&resource_path));
        assert!(!project.lock().unwrap().exists(new_id));

        // Undo delete
        data_manager.undo_transaction().unwrap();
        assert!(project.lock().unwrap().exists_named(&resource_path));
        assert!(project.lock().unwrap().exists(new_id));

        // Undo Create
        data_manager.undo_transaction().unwrap();
        assert!(!project.lock().unwrap().exists_named(&resource_path));
        assert!(!project.lock().unwrap().exists(new_id));

        // Redo Create
        data_manager.redo_transaction().unwrap();
        assert!(project.lock().unwrap().exists_named(&resource_path));
        assert!(project.lock().unwrap().exists(new_id));

        // Redo Delete
        data_manager.redo_transaction().unwrap();
        assert!(!project.lock().unwrap().exists_named(&resource_path));
        assert!(!project.lock().unwrap().exists(new_id));

        drop(data_manager);
    }

    drop(registry);
    drop(project);
    Ok(())
}
