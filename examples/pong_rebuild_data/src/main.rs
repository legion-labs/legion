//! Utility to regenerate the data for the pong demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use clap::{ArgEnum, Parser};
use std::{
    env,
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
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistryOptions, Component, Resource, ResourceId, ResourceTypeAndId,
};
use lgn_data_transaction::BuildManager;
use lgn_graphics_renderer::components::Mesh;
use lgn_math::prelude::*;
use lgn_scripting::ScriptType;
use lgn_tracing::LevelFilter;
use sample_data::{
    offline::{Light, Transform, Visual},
    LightType,
};
use tokio::sync::Mutex;

#[derive(Debug, Copy, Clone, PartialEq, ArgEnum)]
enum CompilersSource {
    InProcess,
    External,
    Remote,
}

#[derive(Parser)]
#[clap(name = "Pong data rebuilder")]
struct Args {
    /// Compile resources remotely.
    #[clap(arg_enum, default_value = "in-process")]
    compilers: CompilersSource,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuardBuilder::default()
        .with_local_sink_max_level(LevelFilter::Info)
        .build();

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
    let mut project = Project::create_with_remote_mock(absolute_project_dir)
        .await
        .expect("failed to create a project");

    let mut resource_registry = ResourceRegistryOptions::new();
    lgn_graphics_data::offline::register_resource_types(&mut resource_registry);
    lgn_scripting::offline::register_resource_types(&mut resource_registry);
    generic_data::offline::register_resource_types(&mut resource_registry);
    sample_data::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let resource_ids = create_offline_data(&mut project, &resource_registry).await;

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    lgn_graphics_data::offline::add_loaders(&mut asset_registry);
    lgn_scripting::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    sample_data::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create();

    let mut compilers_path = env::current_exe().expect("cannot access current_exe");
    compilers_path.pop(); // pop the .exe name

    let compilers = match args.compilers {
        CompilersSource::InProcess => CompilerRegistryOptions::default()
            .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
            .add_compiler(&lgn_compiler_runtime_model::COMPILER_INFO)
            .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO),
        CompilersSource::External => CompilerRegistryOptions::local_compilers(compilers_path),
        CompilersSource::Remote => lgn_data_compiler_remote::compiler_node::remote_compilers(
            compilers_path,
            "lgn://127.0.0.1:2022",
        ),
    };

    let absolute_build_dir = {
        if !build_dir.is_absolute() {
            std::env::current_dir().unwrap().join(&build_dir)
        } else {
            build_dir.clone()
        }
    };
    let data_build = DataBuildOptions::new_with_sqlite_output(&absolute_build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()))
        .asset_registry(asset_registry.clone());

    let mut build_manager = BuildManager::new(
        data_build,
        &project,
        Manifest::default(),
        Manifest::default(),
    )
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
    // visual reference models
    let cube_model_id = create_offline_model(
        project,
        resource_registry,
        "5474d00b-cc10-491a-ba56-be2f5b5de22d",
        Mesh::new_cube(0.5),
        "/scene/models/cube.mod",
    )
    .await;

    let sphere_model_id = create_offline_model(
        project,
        resource_registry,
        "a05e4c89-e85b-4e03-add4-8767b21c1e55",
        Mesh::new_sphere(0.25, 16, 16),
        "/scene/models/sphere.mod",
    )
    .await;

    let ground_path_id = create_offline_entity(
        project,
        resource_registry,
        "63c338c9-0d03-4636-8a17-8f0cba02b618",
        "/scene/ground.ent",
        vec![
            Box::new(Transform {
                position: (0_f32, 0_f32, 0.1_f32).into(),
                scale: (12_f32, 8_f32, 0.01_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0xD0, 0xFF, 0xD0).into(),
                ..Visual::default()
            }),
        ],
        vec![],
    )
    .await;

    let pad_right_script = create_offline_script(
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

    let pad_right_path_id = create_offline_entity(
        project,
        resource_registry,
        "727eef7f-2544-4a46-be99-9aedd44a098e",
        "/scene/pad-right.ent",
        vec![
            Box::new(sample_data::offline::Name {
                name: "Pad Right".to_string(),
            }),
            Box::new(Transform {
                position: (-2.4_f32, 0_f32, 0_f32).into(),
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0x00, 0xFF, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(lgn_scripting::offline::ScriptComponent {
                script_type: ScriptType::Rune,
                input_values: vec!["{entity}".to_string(), "{events}".to_string()],
                entry_fn: "update".to_string(),
                script_id: Some(pad_right_script),
                temp_script: "".to_string(),
            }),
        ],
        vec![],
    )
    .await;

    let pad_left_script = create_offline_script(
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

    let pad_left_path_id = create_offline_entity(
        project,
        resource_registry,
        "719c8d5b-d320-4102-a92a-b3fa5240e140",
        "/scene/pad-left.ent",
        vec![
            Box::new(sample_data::offline::Name {
                name: "Pad Left".to_string(),
            }),
            Box::new(Transform {
                position: (2.4_f32, 0_f32, 0_f32).into(),
                scale: (0.4_f32, 2_f32, 0.4_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0x00, 0x00, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(lgn_scripting::offline::ScriptComponent {
                script_type: ScriptType::Rune,
                input_values: vec!["{entity}".to_string(), "{events}".to_string()],
                entry_fn: "update".to_string(),
                script_id: Some(pad_left_script),
                temp_script: "".to_string(),
            }),
        ],
        vec![],
    )
    .await;

    let ball_script = create_offline_script(
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

    let ball_path_id = create_offline_entity(
        project,
        resource_registry,
        "26b7a335-2d28-489d-882b-f7aae1fb2196",
        "/scene/ball.ent",
        vec![
            Box::new(sample_data::offline::Name {
                name: "Ball".to_string(),
            }),
            Box::new(Transform {
                position: Vec3::ZERO,
                scale: (0.4_f32, 0.4_f32, 0.4_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(sphere_model_id.clone()),
                color: (0xFF, 0x10, 0x40).into(),
                ..Visual::default()
            }),
            Box::new(lgn_scripting::offline::ScriptComponent {
                script_type: ScriptType::Rune,
                input_values: vec![
                    "{entity}".to_string(),
                    "{result}".to_string(),
                    "{entities}".to_string(),
                ],
                entry_fn: "update".to_string(),
                script_id: Some(ball_script),
                temp_script: "".to_string(),
            }),
        ],
        vec![],
    )
    .await;

    let light_id = create_offline_entity(
        project,
        resource_registry,
        "ba1ffa5a-8a54-451d-8d01-f57afce857d3",
        "/scene/light.ent",
        vec![Box::new(Light {
            light_type: LightType::Omnidirectional,
            color: (0xFF, 0xFF, 0xEF).into(),
            radiance: 40_f32,
            enabled: true,
            ..Light::default()
        })],
        vec![],
    )
    .await;

    // scene
    let scene_id = create_offline_entity(
        project,
        resource_registry,
        "29b8b0d0-ee1e-4792-aca2-3b3a3ce63916",
        "/scene.ent",
        vec![Box::new(Transform {
            position: (0_f32, 0_f32, 4_f32).into(),
            scale: (0.5_f32, 0.5_f32, 0.5_f32).into(),
            ..Transform::default()
        })],
        vec![
            ground_path_id,
            pad_right_path_id,
            pad_left_path_id,
            ball_path_id,
            light_id,
        ],
    )
    .await;

    vec![scene_id.source_resource()]
}

async fn create_offline_entity(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    resource_id: &str,
    resource_path: &str,
    components: Vec<Box<dyn Component>>,
    children: Vec<ResourcePathId>,
) -> ResourcePathId {
    let mut resources = resource_registry.lock().await;
    let id = ResourceTypeAndId {
        kind: sample_data::offline::Entity::TYPE,
        id: ResourceId::from_str(resource_id).unwrap(),
    };
    let handle = resources.new_resource(id.kind).unwrap();

    let entity = handle
        .get_mut::<sample_data::offline::Entity>(&mut resources)
        .unwrap();
    entity.components.extend(components.into_iter());
    entity.children.extend(children.into_iter());

    project
        .add_resource_with_id(
            resource_path.into(),
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
}

async fn create_offline_model(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    resource_id: &str,
    mesh: Mesh,
    resource_path: &str,
) -> ResourcePathId {
    let mut resources = resource_registry.lock().await;
    let id = ResourceTypeAndId {
        kind: lgn_graphics_data::offline::Model::TYPE,
        id: ResourceId::from_str(resource_id).unwrap(),
    };
    let handle = resources.new_resource(id.kind).unwrap();

    let model = handle
        .get_mut::<lgn_graphics_data::offline::Model>(&mut resources)
        .unwrap();
    let mesh = lgn_graphics_data::offline::Mesh {
        positions: mesh.positions,
        normals: mesh.normals.unwrap(),
        tangents: mesh.tangents.unwrap(),
        tex_coords: mesh.tex_coords.unwrap(),
        indices: mesh.indices.unwrap(),
        colors: mesh
            .colors
            .map(|colors| colors.iter().map(|color| (*color).into()).collect())
            .unwrap(),
        material: None,
    };
    model.meshes.push(mesh);

    project
        .add_resource_with_id(
            resource_path.into(),
            lgn_graphics_data::offline::Model::TYPENAME,
            id.kind,
            id.id,
            handle,
            &mut resources,
        )
        .await
        .unwrap();
    let path: ResourcePathId = id.into();
    path.push(lgn_graphics_data::runtime::Model::TYPE)
}

async fn create_offline_script(
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
