use std::path::PathBuf;
use std::sync::Arc;

use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistryOptions};
use lgn_data_transaction::{BuildManager, DataManager};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let project_dir = PathBuf::from("examples/pong/data");
    let build_dir = project_dir.join("temp");
    
    std::fs::create_dir_all(&build_dir).unwrap();

    let project = Project::create_new(project_dir)
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
    let asset_registry = asset_registry.create();

    let compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_debugcube::COMPILER_INFO)
        .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO);

    let options = DataBuildOptions::new(&build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()))
        .asset_registry(asset_registry.clone());

    let build_manager = BuildManager::new(options, &project, Manifest::default())
        .await
        .unwrap();
    let project = Arc::new(Mutex::new(project));

    let _data_manager = Arc::new(Mutex::new(DataManager::new(
        project,
        resource_registry,
        asset_registry,
        build_manager,
    )));
}
