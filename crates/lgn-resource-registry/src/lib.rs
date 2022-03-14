//! The resource registry plugin provides loading of offline resources.

// crate-specific lint exceptions:
//#![allow()]

mod settings;

use std::sync::Arc;

use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistry, AssetRegistryScheduling};
use lgn_data_transaction::{BuildManager, SelectionManager, TransactionManager};
use lgn_ecs::prelude::*;
pub use settings::ResourceRegistrySettings;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ResourceRegistryPlugin {}

pub struct ResourceRegistryCreated {}

/// Label to use for scheduling systems that require the `ResourceRegistry`
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum ResourceRegistryPluginScheduling {
    /// AssetRegistry has been created
    ResourceRegistryCreated,
}

impl Plugin for ResourceRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionManager::create());
        app.add_startup_system_to_stage(StartupStage::PreStartup, Self::pre_setup);
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::post_setup
                .exclusive_system()
                .after(AssetRegistryScheduling::AssetRegistryCreated)
                .label(ResourceRegistryPluginScheduling::ResourceRegistryCreated),
        );
        app.add_startup_system(register_resource_dir);
    }
}

impl ResourceRegistryPlugin {
    fn pre_setup(mut commands: Commands<'_, '_>) {
        let registry_options = ResourceRegistryOptions::new();
        commands.insert_resource(registry_options);
    }

    fn post_setup(world: &mut World) {
        let registry_options = world.remove_resource::<ResourceRegistryOptions>().unwrap();
        let registry = registry_options.create_async_registry();

        let settings = world.get_resource::<ResourceRegistrySettings>().unwrap();
        let project_dir = settings.root_folder.clone();
        let build_dir = project_dir.join("temp");

        let async_rt = world.get_resource::<TokioAsyncRuntime>().unwrap();
        let asset_registry = world.get_resource::<Arc<AssetRegistry>>().unwrap();
        let intermediate_manifest = Manifest::default();
        let runtime_manifest = world.get_resource::<Manifest>().unwrap();
        let selection_manager = world.get_resource::<Arc<SelectionManager>>().unwrap();

        let transaction_manager = async_rt.block_on(async move {
            sample_data_compiler::raw_loader::build_offline(&project_dir, false).await;

            let project = {
                if let Ok(project) = Project::open(&project_dir).await {
                    project
                } else {
                    let mut project =
                        Project::create(&project_dir, settings.source_control_path.clone())
                            .await
                            .unwrap();
                    project.sync_latest().await.unwrap();
                    project
                }
            };

            let compilers = lgn_ubercompiler::create();

            let build_options = DataBuildOptions::new(&build_dir, compilers)
                .content_store(&ContentStoreAddr::from(build_dir.as_path()))
                .manifest(intermediate_manifest.clone());

            let build_manager = BuildManager::new(
                build_options,
                &project,
                runtime_manifest.clone(),
                intermediate_manifest.clone(),
            )
            .await
            .expect("the editor requires valid build manager");

            Arc::new(Mutex::new(TransactionManager::new(
                Arc::new(Mutex::new(project)),
                registry,
                asset_registry.clone(),
                build_manager,
                selection_manager.clone(),
            )))
        });

        {
            let async_rt = world
                .get_resource::<TokioAsyncRuntime>()
                .expect("async plugin did not provide tokio runtime");
            let transaction_manager = transaction_manager.clone();
            async_rt.start_detached(async move {
                let mut transaction_manager = transaction_manager.lock().await;
                transaction_manager.load_all_resources().await;
            });
        }

        world.insert_resource(transaction_manager);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn register_resource_dir(
    settings: Res<'_, ResourceRegistrySettings>,
    mut registry: NonSendMut<'_, lgn_data_runtime::AssetRegistryOptions>,
) {
    let project_dir = settings.root_folder.join("offline");
    registry.add_device_dir_mut(project_dir);
}
