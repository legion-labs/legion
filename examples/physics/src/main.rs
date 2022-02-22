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
    // ground
    let ground_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("63c338c9-0d03-4636-8a17-8f0cba02b618").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(sample_data::offline::Name {
            name: "Ground".to_string(),
        }));
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (0_f32, 0_f32, -0.1_f32).into(),
                rotation: Quat::default(),
                scale: (12_f32, 8_f32, 0.01_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (208, 255, 208).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/Ground".into(),
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
            id: ResourceId::from_str("29b8b0d0-ee1e-4792-aca2-3b3a3ce63916").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();

        entity.children.push(ground_path_id);

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
