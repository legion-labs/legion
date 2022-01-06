//! The resource registry plugin provides loading of offline resources.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

mod settings;

use std::sync::Arc;

use lgn_app::prelude::*;
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_offline::resource::{Project, ResourceRegistryOptions};
use lgn_data_runtime::{manifest::Manifest, AssetRegistry, AssetRegistryScheduling};
use lgn_data_transaction::{BuildManager, DataManager};
use lgn_ecs::prelude::*;
use lgn_tasks::IoTaskPool;
pub use settings::ResourceRegistrySettings;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ResourceRegistryPlugin {}

impl Plugin for ResourceRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, Self::pre_setup);
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::post_setup
                .exclusive_system()
                .after(AssetRegistryScheduling::AssetRegistryCreated),
        );
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

        let project = Project::open(&project_dir).expect("unable to open project dir");

        let project = Arc::new(Mutex::new(project));

        let compilers = lgn_ubercompiler::create();

        let build_options = DataBuildOptions::new(&build_dir, compilers)
            .content_store(&ContentStoreAddr::from(build_dir.as_path()));

        let manifest = world.get_resource::<Manifest>().unwrap();
        let build_manager = BuildManager::new(build_options, &project_dir, manifest.clone())
            .expect("the editor requires valid build manager");

        let asset_registry = world.get_resource::<Arc<AssetRegistry>>().unwrap();
        let data_manager = Arc::new(Mutex::new(DataManager::new(
            project,
            registry,
            asset_registry.clone(),
            build_manager,
        )));

        {
            let data_manager = data_manager.clone();
            let io_task_pool = world.get_resource::<IoTaskPool>().unwrap();
            io_task_pool
                .spawn(async move {
                    let mut data_manager = data_manager.lock().await;
                    data_manager.load_all_resources().await;
                })
                .detach();
        }

        world.insert_resource(data_manager);
    }
}
