//! The resource registry plugin provides loading of offline resources.

// crate-specific lint exceptions:
//#![allow()]

pub mod settings;

use std::sync::Arc;

use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
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

        let settings = world.resource::<ResourceRegistrySettings>();
        let project_dir = settings.root_folder.clone();

        let async_rt = world.resource::<TokioAsyncRuntime>();
        let asset_registry = world.resource::<Arc<AssetRegistry>>();
        let intermediate_manifest = Manifest::default();
        let runtime_manifest = world.resource::<Manifest>();
        let selection_manager = world.resource::<Arc<SelectionManager>>();

        let transaction_manager = async_rt.block_on(async move {
            let content_provider = Arc::new(
                lgn_content_store2::Config::load_and_instantiate_persistent_provider()
                    .await
                    .unwrap(),
            );

            sample_data_compiler::raw_loader::build_offline(
                &project_dir,
                settings.source_control_path.clone(),
                Arc::clone(&content_provider),
                false,
            )
            .await;

            let project = {
                if let Ok(project) =
                    Project::open(&project_dir, Arc::clone(&content_provider)).await
                {
                    project
                } else {
                    let mut project = Project::create(
                        &project_dir,
                        settings.source_control_path.clone(),
                        content_provider,
                    )
                    .await
                    .unwrap();
                    project.sync_latest().await.unwrap();
                    project
                }
            };

            let mut compiler_dir = std::env::current_exe().expect("cannot access current_exe");
            compiler_dir.pop(); // pop the .exe name
            let compilers = match &settings.compilation_mode {
                settings::CompilationMode::InProcess => lgn_ubercompiler::create(),
                settings::CompilationMode::External => {
                    CompilerRegistryOptions::local_compilers(compiler_dir)
                }
                settings::CompilationMode::Remote { url } => {
                    lgn_data_compiler_remote::compiler_node::remote_compilers(compiler_dir, url)
                }
            };

            let build_options = DataBuildOptions::new(
                DataBuildOptions::output_db_path(
                    &settings.build_output_db_addr,
                    project_dir.as_path(),
                    DataBuild::version(),
                ),
                settings.content_store_addr.clone(),
                compilers,
            )
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
