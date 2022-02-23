//! Utility to regenerate the data for the physics demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::{
    resource::{Project, ResourceRegistry, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions};
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
use lgn_data_transaction::BuildManager;
use lgn_math::prelude::*;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default().unwrap();

    let project_dir = PathBuf::from("examples/physics/data");

    clean_folders(&project_dir);

    let build_dir = project_dir.join("temp");
    std::fs::create_dir_all(&build_dir).unwrap();

    let absolute_project_dir = {
        if !project_dir.is_absolute() {
            std::env::current_dir().unwrap().join(&project_dir)
        } else {
            project_dir.clone()
        }
    };
    let mut project = Project::create_new(absolute_project_dir)
        .await
        .expect("failed to create a project");

    let mut resource_registry = ResourceRegistryOptions::new();
    sample_data::offline::register_resource_types(&mut resource_registry);
    generic_data::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let resource_ids = create_offline_data(&mut project, &resource_registry).await;

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    sample_data::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create();

    let compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO);

    let data_build = DataBuildOptions::new(&build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()))
        .asset_registry(asset_registry.clone());

    let mut build_manager = BuildManager::new(data_build, &project, Manifest::default())
        .await
        .unwrap();

    for id in resource_ids {
        build_manager.build_all_derived(id, &project).await.unwrap();
    }

    let runtime_dir = project_dir.join("runtime");
    std::fs::create_dir(&runtime_dir).unwrap();
    let runtime_manifest_path = runtime_dir.join("game.manifest");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(runtime_manifest_path)
        .expect("open file");

    serde_json::to_writer_pretty(file, &build_manager.get_manifest()).expect("to write manifest");
}

fn clean_folders(project_dir: impl AsRef<Path>) {
    let mut path = project_dir.as_ref().to_owned();

    let mut clean = |sub_path| {
        path.push(sub_path);
        if path.exists() {
            let remove = if path.is_dir() {
                std::fs::remove_dir_all
            } else {
                std::fs::remove_file
            };
            remove(path.as_path()).unwrap_or_else(|_| panic!("Cannot delete {:?}", path));
        }
        path.pop();
    };

    clean("remote");
    clean("offline");
    clean("runtime");
    clean("temp");
    clean("project.index");
}

async fn create_offline_data(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
) -> Vec<ResourceTypeAndId> {
    let ground_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("8859bf63-f187-4aa3-afd5-425130c7ba04").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (0_f32, -1_f32, 0_f32).into(),
                rotation: Quat::default(),
                scale: (12_f32, 0.1_f32, 12_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0x10, 0x10, 0x55).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/ground.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(sample_data::runtime::Entity::TYPE)
    };

    let box_a_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("8e04418d-ca9a-4e4a-a0ea-68e74d8c10d0").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (-1_f32, 0.1_f32, 1_f32).into(),
                rotation: Quat::from_rotation_z(0.3_f32),
                scale: (1_f32, 1_f32, 1_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0xFF, 0xFF, 0x20).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/box-a.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(sample_data::runtime::Entity::TYPE)
    };

    let box_b_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("b355fa97-97ee-44f9-afbf-1c2920ce5064").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (-1_f32, 1.3_f32, 1.2_f32).into(),
                rotation: Quat::from_rotation_x(-0.2_f32),
                scale: (1_f32, 1_f32, 1_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0xFF, 0x20, 0xFF).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/box-b.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(sample_data::runtime::Entity::TYPE)
    };

    let box_c_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("6715d493-d7d4-4155-bc18-0e1795c53580").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (-0.5_f32, 0_f32, 0.5_f32).into(),
                rotation: Quat::from_rotation_y(1_f32),
                scale: (1_f32, 1_f32, 1_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0x20, 0xFF, 0xFF).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/box-c.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(sample_data::runtime::Entity::TYPE)
    };

    let scene_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("09f7380d-51b2-4061-9fe4-52ceccce55e7").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();

        entity.children.push(box_a_id);
        entity.children.push(box_b_id);
        entity.children.push(box_c_id);
        entity.children.push(ground_id);

        project
            .add_resource_with_id(
                "/scene.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        id
    };

    vec![scene_id]
}
