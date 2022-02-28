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
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (0_f32, 0_f32, 0.1_f32).into(),
                rotation: Quat::IDENTITY,
                scale: (12_f32, 8_f32, 0.01_f32).into(),
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::Visual {
                color: (208, 255, 208).into(),
                ..sample_data::offline::Visual::default()
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

    // pad right
    let pad_right_script = build_script(
        project,
        resource_registry,
        "e93151b6-3635-4a30-9f3e-e6052929d85a",
        ScriptType::Rune,
        "/scene/pad_right_script",
        r#"
const MOUSE_DELTA_SCALE = 200.0;

pub fn update(entity, events) {
    let delta_x = events.mouse_motion.x / MOUSE_DELTA_SCALE;
    if let Some(transform) = entity.transform {
        transform.translation.y += delta_x;
        transform.translation.clamp_y(-2.0, 2.0);
    }
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
                rotation: Quat::IDENTITY,
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::Visual {
                color: (0, 255, 255).into(),
                ..sample_data::offline::Visual::default()
            }));

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec!["{entity}".to_string(), "{events}".to_string()],
            entry_fn: "update".to_string(),
            script_id: Some(pad_right_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        project
            .add_resource_with_id(
                "/scene/pad-right.ent".into(),
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
        r#"
const MOUSE_DELTA_SCALE = 200.0;

pub fn update(entity, events) {
    let delta_x = events.mouse_motion.x / MOUSE_DELTA_SCALE;
    if let Some(transform) = entity.transform {
        transform.translation.y -= delta_x;
        transform.translation.clamp_y(-2.0, 2.0);
    }
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
                rotation: Quat::IDENTITY,
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::Visual {
                color: (0, 0, 255).into(),
                ..sample_data::offline::Visual::default()
            }));

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec!["{entity}".to_string(), "{events}".to_string()],
            entry_fn: "update".to_string(),
            script_id: Some(pad_left_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        project
            .add_resource_with_id(
                "/scene/pad-left.ent".into(),
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
    let ball_script = build_script(
        project,
        resource_registry,
        "6ec6db36-6d09-4bb2-b9a8-b85c25e5b2c0",
        ScriptType::Rune,
        "/scene/ball_script",
        r#"
use lgn_math::{normalize2, random};

const VELOCITY = 0.7;

struct Vec2 {
    x,
    y,
}

pub fn update(entity, last_result, entities) {
    let ball_direction = if last_result is unit {
        let v = Vec2 {
            x: random() - 0.5,
            y: random() - 0.5,
        };
        if let (vx, vy) = normalize2(v.x, v.y) {
            v.x = vx * 0.1;
            v.y = vy * 0.1;
        }
        v
    } else {
        last_result
    };

    let transform = entity.transform.unwrap();
    let position = transform.translation;

    if position.x < -3.0 || position.x > 3.0 {
        ball_direction.x = -ball_direction.x;
    }
    if position.y < -2.0 || position.y > 2.0 {
        ball_direction.y = -ball_direction.y;
    }

    position.clamp_x(-3.0, 3.0);
    position.clamp_y(-2.0, 2.0);

    // update paddles
    let left_paddle = 0.0;
    if let Some(entity) = entities["Pad Left"] {
        if let Some(transform) = entity.transform {
            left_paddle = transform.translation.y;
        }
    }
    let right_paddle = 0.0;
    if let Some(entity) = entities["Pad Right"] {
        if let Some(transform) = entity.transform {
            right_paddle = transform.translation.y;
        }
    }

    // check for collision with paddles (dimensions = 0.2 x 1.0 x 0.2)
    // Note: x-axis is inverted so values decrease towards the right
    let new_position = Vec2 {
        x: position.x + VELOCITY * ball_direction.x,
        y: position.y + VELOCITY * ball_direction.y,
    };

    if ball_direction.x > 0.0 {
        // moving left
        if (position.x < 2.3
            && new_position.x >= 2.3
            && position.y > left_paddle - 0.5
            && position.y < left_paddle + 0.5)
            || (position.x < -2.5
                && new_position.x >= -2.5
                && position.y > right_paddle - 0.5
                && position.y < right_paddle + 0.5)
        {
            ball_direction.x = -ball_direction.x;
        }
    } else {
        // moving right
        if (position.x > -2.3
            && new_position.x <= -2.3
            && position.y > right_paddle - 0.5
            && position.y < right_paddle + 0.5)
            || (position.x > 2.5
                && new_position.x <= 2.5
                && position.y > left_paddle - 0.5
                && position.y < left_paddle + 0.5)
        {
            ball_direction.x = -ball_direction.x;
        }
    }

    if ball_direction.y > 0.0 {
        // moving up
        let left_bottom = left_paddle - 0.5;
        let right_bottom = right_paddle - 0.5;
        if (position.y < left_bottom
            && new_position.y >= left_bottom
            && position.x > 2.3
            && position.x < 2.5)
            || (position.y < right_bottom
                && new_position.y >= right_bottom
                && position.x < -2.3
                && position.x > -2.5)
        {
            ball_direction.y = -ball_direction.y;
        }
    } else {
        // moving down
        let left_top = left_paddle + 0.5;
        let right_top = right_paddle + 0.5;
        if (position.y > left_top
            && new_position.y <= left_top
            && position.x > 2.3
            && position.x < 2.5)
            || (position.y > right_top
                && new_position.y <= right_top
                && position.x < -2.3
                && position.x > -2.5)
        {
            ball_direction.y = -ball_direction.y;
        }
    }    

    position.x += VELOCITY * ball_direction.x;
    position.y += VELOCITY * ball_direction.y;

    ball_direction
}"#,
    )
    .await;
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
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: (0.4_f32, 0.4_f32, 0.4_f32).into(),
            }));
        entity
            .components
            .push(Box::new(sample_data::offline::Visual {
                color: (255, 16, 64).into(),
                ..sample_data::offline::Visual::default()
            }));

        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: ScriptType::Rune,
            input_values: vec![
                "{entity}".to_string(),
                "{result}".to_string(),
                "{entities}".to_string(),
            ],
            entry_fn: "update".to_string(),
            script_id: Some(ball_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);

        project
            .add_resource_with_id(
                "/scene/ball.ent".into(),
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

        // move back scene, and scale down
        entity
            .components
            .push(Box::new(sample_data::offline::Transform {
                position: (0_f32, 0_f32, 4_f32).into(),
                rotation: Quat::IDENTITY,
                scale: (0.5_f32, 0.5_f32, 0.5_f32).into(),
            }));

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
