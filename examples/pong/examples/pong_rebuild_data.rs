use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::{ContentStoreAddr, HddContentStore};
//use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::{
    resource::{Project, ResourceRegistry, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions};
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
//use lgn_data_transaction::BuildManager;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let project_dir = {
        let project_dir = PathBuf::from("examples/pong/data");
        if !project_dir.is_absolute() {
            std::env::current_dir().unwrap().join(project_dir)
        } else {
            project_dir
        }
    };

    clean_folders(&project_dir);

    let build_dir = project_dir.join("temp");

    std::fs::create_dir_all(&build_dir).unwrap();

    let mut project = Project::create_new(project_dir)
        .await
        .expect("failed to create a project");

    let mut resource_registry = ResourceRegistryOptions::new();
    sample_data::offline::register_resource_types(&mut resource_registry);
    generic_data::offline::register_resource_types(&mut resource_registry);
    lgn_scripting::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    sample_data::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    lgn_scripting::offline::add_loaders(&mut asset_registry);
    let _asset_registry = asset_registry.create();

    let _compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_debugcube::COMPILER_INFO)
        .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO);

    // let options = DataBuildOptions::new(&build_dir, compilers)
    //     .content_store(&ContentStoreAddr::from(build_dir.as_path()))
    //     .asset_registry(asset_registry.clone());

    // let build_manager = BuildManager::new(options, &project, Manifest::default())
    //     .await
    //     .unwrap();

    create_offline_data(&mut project, &resource_registry).await;
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
) {
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
                id,
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
                id,
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
                id,
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
                id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        let path: ResourcePathId = id.into();
        path.push(generic_data::runtime::DebugCube::TYPE)
    };

    // scene
    let _scene_id = {
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
        project
            .add_resource_with_id(
                "/scene.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        id
    };
}
