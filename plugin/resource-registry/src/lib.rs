//! The resource registry plugin provides loading of offline resources.
//!

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

use legion_app::Plugin;
use legion_data_offline::resource::{Project, ResourceRegistryOptions};
use legion_data_runtime::AssetRegistry;
use legion_data_transaction::DataManager;
use legion_tasks::IoTaskPool;
use sample_data_offline as offline_data;
pub use settings::ResourceRegistrySettings;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ResourceRegistryPlugin {}

impl Plugin for ResourceRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(settings) = app.world.get_resource::<ResourceRegistrySettings>() {
            if let Ok(project) = Project::open(&settings.root_folder) {
                // register resource types
                let mut registry = ResourceRegistryOptions::new();
                registry = offline_data::register_resource_types(registry);
                registry = legion_graphics_offline::register_resource_types(registry);
                registry = generic_data_offline::register_resource_types(registry);
                let registry = registry.create_async_registry();
                let project = Arc::new(Mutex::new(project));

                let asset_registry = app
                    .world
                    .get_resource::<Arc<AssetRegistry>>()
                    .expect("the editor plugin requires AssetRegistry resource");

                let data_manager = Arc::new(Mutex::new(DataManager::new(
                    project,
                    registry,
                    asset_registry.clone(),
                )));

                let task_pool = app
                    .world
                    .get_resource::<IoTaskPool>()
                    .expect("IoTaskPool is not available, missing CorePlugin?");

                {
                    let data_manager = data_manager.clone();
                    task_pool
                        .spawn(async move {
                            let mut data_manager = data_manager.lock().await;
                            data_manager.load_all_resources().await;
                        })
                        .detach();
                }

                app.insert_resource(data_manager);
            }
        }
    }
}
