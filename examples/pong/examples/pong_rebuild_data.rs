use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use lgn_content_store::{ContentStoreAddr, HddContentStore};
//use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions};
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
//use lgn_data_transaction::BuildManager;

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
    lgn_scripting::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    sample_data::offline::add_loaders(&mut asset_registry);
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

    // ball
    let _ball_id = {
        let mut resources = resource_registry.lock().await;
        let ball_id = ResourceTypeAndId {
            kind: generic_data::offline::DebugCube::TYPE,
            id: ResourceId::from_str("95114898-37a9-51c4-53ad-e7d62c10935b").unwrap(),
        };
        let ball_handle = resources.new_resource(ball_id.kind).unwrap();
        let ball_entity = ball_handle
            .get_mut::<generic_data::offline::DebugCube>(&mut resources)
            .unwrap();
        ball_entity.color = (255, 16, 64).into();
        ball_entity.mesh_id = 8;
        ball_entity.name = "Ball".to_string();
        ball_entity.rotation_speed = (0.1_f32, 0_f32, 0_f32).into();
        ball_entity.scale = (0.4_f32, 0.4_f32, 0.4_f32).into();
        project
            .add_resource_with_id(
                "/scene/Ball".into(),
                generic_data::offline::DebugCube::TYPENAME,
                ball_id.kind,
                ball_id,
                ball_handle,
                &mut resources,
            )
            .await
            .unwrap();
        ball_id
    };
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
