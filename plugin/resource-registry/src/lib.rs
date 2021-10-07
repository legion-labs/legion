//! The resource registry plugin provides loading of offline resources.
//!

// BEGIN - Legion Labs lints v0.5
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]

mod resource_handles;
mod settings;

use legion_data_runtime::ResourceId;
use resource_handles::ResourceHandles;
pub use settings::ResourceRegistrySettings;

use legion_app::Plugin;
use legion_data_offline::resource::{Project, ResourceRegistry, ResourceRegistryOptions};
use legion_ecs::prelude::*;
use sample_data_compiler::offline_data;

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
                let registry = registry.create_registry();

                app.insert_resource(project)
                    .insert_resource(registry)
                    .insert_resource(ResourceHandles::default())
                    .add_startup_system(Self::setup);
            }
        }
    }
}

impl ResourceRegistryPlugin {
    fn setup(
        project: ResMut<'_, Project>,
        mut registry: ResMut<'_, ResourceRegistry>,
        mut resource_handles: ResMut<'_, ResourceHandles>,
    ) {
        for resource_id in project.resource_list() {
            Self::load_resource(&project, &mut registry, &mut resource_handles, resource_id);
        }

        drop(project);
    }

    fn load_resource(
        project: &ResMut<'_, Project>,
        registry: &mut ResMut<'_, ResourceRegistry>,
        resource_handles: &mut ResMut<'_, ResourceHandles>,
        resource_id: ResourceId,
    ) {
        if let Some(_handle) = resource_handles.get(resource_id) {
            // already in resource list
            println!("New reference to loaded resource: {}", resource_id);
        } else {
            match project.load_resource(resource_id, registry) {
                Ok(handle) => {
                    println!("Loaded resource: {}", resource_id);
                    resource_handles.insert(resource_id, handle);
                }
                Err(err) => eprintln!("Failed to load resource {}: {}", resource_id, err),
            }
        }
    }
}
