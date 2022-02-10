//! Utility to regenerate the data for the pong demo.
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
use lgn_scripting::ScriptType;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default().unwrap();

    let project_dir = PathBuf::from("examples/pong/data");

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
    lgn_scripting::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let resource_ids = create_offline_data(&mut project, &resource_registry).await;

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    sample_data::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    lgn_scripting::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create();

    let compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO);

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

async fn build_script(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    guid: &str,
    script_type: ScriptType,
    file_name: &str,
    script_text: &str,
) -> ResourcePathId {
    let mut resources = resource_registry.lock().await;
    let id = ResourceTypeAndId {
        kind: lgn_scripting::offline::Script::TYPE,
        id: ResourceId::from_str(guid).unwrap(),
    };
    let handle = resources.new_resource(id.kind).unwrap();
    let script = handle
        .get_mut::<lgn_scripting::offline::Script>(&mut resources)
        .unwrap();
    script.script_type = script_type;
    script.script = script_text.to_string();
    project
        .add_resource_with_id(
            file_name.into(),
            lgn_scripting::offline::Script::TYPENAME,
            id.kind,
            id.id,
            &handle,
            &mut resources,
        )
        .await
        .unwrap();
    let path: ResourcePathId = id.into();
    path.push(lgn_scripting::runtime::Script::TYPE)
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

    // pad right
    let pad_right_script = build_script(
        project,
        resource_registry,
        "e93151b6-3635-4a30-9f3e-e6052929d85a",
        ScriptType::Rune,
        "/scene/pad_right_script",
        r#"use lgn_math::Vec2;

const MOUSE_DELTA_SCALE = 200.0;

pub fn move_right_paddle(delta) {
    delta.x /= MOUSE_DELTA_SCALE;
    println!("[Pad right] delta: {}", delta.x);
}"#,
    )
    .await;
    let pad_right_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("727eef7f-2544-4a46-be99-9aedd44a098e").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(sample_data::offline::Name {
            name: "Pad Right".to_string(),
        }));
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (-2.4_f32, 0_f32, 0_f32).into(),
                rotation: Quat::default(),
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0, 255, 255).into(),
                mesh: None,
            }));

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec!["mouse_motion.delta".to_string()],
            entry_fn: "move_right_paddle".to_string(),
            script_id: Some(pad_right_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        project
            .add_resource_with_id(
                "/scene/Pad Right".into(),
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

    // pad left
    let pad_left_script = build_script(
        project,
        resource_registry,
        "968c4926-ae75-4955-81c8-7b7e395d0d3b",
        ScriptType::Rune,
        "/scene/pad_left_script",
        r#"use lgn_math::Vec2;

const MOUSE_DELTA_SCALE = 200.0;

pub fn move_left_paddle(delta) {
delta.x /= MOUSE_DELTA_SCALE;
println!("[Pad left] delta: {}", delta.x);
}"#,
    )
    .await;
    let pad_left_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("719c8d5b-d320-4102-a92a-b3fa5240e140").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(sample_data::offline::Name {
            name: "Pad Left".to_string(),
        }));
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (2.4_f32, 0_f32, 0_f32).into(),
                rotation: Quat::default(),
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Cube,
                color: (0, 0, 255).into(),
                mesh: None,
            }));

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec!["mouse_motion.delta".to_string()],
            entry_fn: "move_left_paddle".to_string(),
            script_id: Some(pad_left_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        project
            .add_resource_with_id(
                "/scene/Pad Left".into(),
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

    // ball
    let ball_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("26b7a335-2d28-489d-882b-f7aae1fb2196").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(sample_data::offline::Name {
            name: "Ball".to_string(),
        }));
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: Vec3::default(),
                rotation: Quat::default(),
                scale: (0.4_f32, 0.4_f32, 0.4_f32).into(),
                apply_to_children: false,
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::StaticMesh {
                mesh_id: lgn_graphics_data::DefaultMeshType::Sphere,
                color: (255, 16, 64).into(),
                mesh: None,
            }));

        project
            .add_resource_with_id(
                "/scene/Ball".into(),
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

    // scene
    let scene_script = build_script(
        project,
        resource_registry,
        "f7e3757c-22b1-44af-a8d3-5ae080c4fef1",
        ScriptType::Rune,
        "/scene/scene_script",
        r#"pub fn update() {}"#,
    )
    .await;

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

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec![],
            entry_fn: "update".to_string(),
            script_id: Some(scene_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        entity.children.push(ground_path_id);
        entity.children.push(pad_right_path_id);
        entity.children.push(pad_left_path_id);
        entity.children.push(ball_path_id);

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
