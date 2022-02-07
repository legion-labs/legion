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
        .add_compiler(&lgn_compiler_debugcube::COMPILER_INFO)
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

// Script
async fn build_script(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    guid: &str,
    script_type: usize,
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
            kind: generic_data::offline::DebugCube::TYPE,
            id: ResourceId::from_str("63c338c9-0d03-4636-8a17-8f0cba02b618").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();
        let debug_cube = handle
            .get_mut::<generic_data::offline::DebugCube>(&mut resources)
            .unwrap();
        debug_cube.color = (208, 255, 208).into();
        debug_cube.name = "Ground".to_string();
        debug_cube.position = (0_f32, 0_f32, -0.1_f32).into();
        debug_cube.scale = (12_f32, 8_f32, 0.01_f32).into();
        project
            .add_resource_with_id(
                "/scene/Ground".into(),
                generic_data::offline::DebugCube::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(generic_data::runtime::DebugCube::TYPE)
    };

    // pad right
    let pad_right_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: generic_data::offline::DebugCube::TYPE,
            id: ResourceId::from_str("727eef7f-2544-4a46-be99-9aedd44a098e").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();
        let debug_cube = handle
            .get_mut::<generic_data::offline::DebugCube>(&mut resources)
            .unwrap();
        debug_cube.color = (0, 255, 255).into();
        debug_cube.name = "Pad Right".to_string();
        debug_cube.position = (-2.4_f32, 0_f32, 0_f32).into();
        debug_cube.scale = (0.4_f32, 2_f32, 0.4_f32).into();
        project
            .add_resource_with_id(
                "/scene/Pad Right".into(),
                generic_data::offline::DebugCube::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(generic_data::runtime::DebugCube::TYPE)
    };

    // pad left
    let pad_left_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: generic_data::offline::DebugCube::TYPE,
            id: ResourceId::from_str("719c8d5b-d320-4102-a92a-b3fa5240e140").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();
        let debug_cube = handle
            .get_mut::<generic_data::offline::DebugCube>(&mut resources)
            .unwrap();
        debug_cube.color = (0, 0, 255).into();
        debug_cube.name = "Pad Left".to_string();
        debug_cube.position = (2.4_f32, 0_f32, 0_f32).into();
        debug_cube.scale = (0.4_f32, 2_f32, 0.4_f32).into();
        project
            .add_resource_with_id(
                "/scene/Pad Left".into(),
                generic_data::offline::DebugCube::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(generic_data::runtime::DebugCube::TYPE)
    };

    // ball
    let ball_path_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: generic_data::offline::DebugCube::TYPE,
            id: ResourceId::from_str("26b7a335-2d28-489d-882b-f7aae1fb2196").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();
        let debug_cube = handle
            .get_mut::<generic_data::offline::DebugCube>(&mut resources)
            .unwrap();
        debug_cube.color = (255, 16, 64).into();
        debug_cube.mesh_id = 8;
        debug_cube.name = "Ball".to_string();
        debug_cube.rotation_speed = (0.1_f32, 0_f32, 0_f32).into();
        debug_cube.scale = (0.4_f32, 0.4_f32, 0.4_f32).into();
        project
            .add_resource_with_id(
                "/scene/Ball".into(),
                generic_data::offline::DebugCube::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(generic_data::runtime::DebugCube::TYPE)
    };

    // Mun script
    let _mun_script = build_script(
        project,
        resource_registry,
        "d46d7e71-27bc-4516-9e85-074f5431d29c",
        1,
        "/scene/mun_script",
        r#"pub fn fibonacci(n: i64) -> i64 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }"#,
    )
    .await;

    // Rune script
    let rune_script = build_script(
        project,
        resource_registry,
        "f7e3757c-22b1-44af-a8d3-5ae080c4fef1",
        2,
        "/scene/rune_script",
        r#"pub fn fibonacci(n) {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }"#,
    )
    .await;

    // Rhai script
    let _rhai_script = build_script(
        project,
        resource_registry,
        "44823dbf-597c-4e5b-90e9-10868ed7cefe",
        3,
        "/scene/rhai_script",
        r#"fn fibonacci(n) {
            if n < 2 {
                n
            } else {
                fibonacci(n-1) + fibonacci(n-2)
            }
        }"#,
    )
    .await;

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
        entity.children.push(ground_path_id);
        entity.children.push(pad_right_path_id);
        entity.children.push(pad_left_path_id);
        entity.children.push(ball_path_id);
        let script_component = Box::new(lgn_scripting::offline::ScriptComponent {
            script_type: 2, // Rune
            input_values: vec!["10".to_string()],
            entry_fn: "fibonacci".to_string(),
            script_id: Some(rune_script),
            temp_script: "".to_string(),
        });
        entity.components.push(script_component);
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
