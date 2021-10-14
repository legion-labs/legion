//! The asset registry plugin provides loading of runtime assets.
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

mod asset_entities;
mod asset_handles;
mod asset_to_ecs;
mod loading_states;
mod settings;

use asset_entities::AssetToEntityMap;
use asset_handles::AssetHandles;
use asset_to_ecs::load_ecs_asset;
use loading_states::{AssetLoadingStates, LoadingState};
pub use settings::AssetRegistrySettings;

use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{manifest::Manifest, AssetRegistry, AssetRegistryOptions};
use legion_ecs::prelude::*;
use sample_data_compiler::runtime_data;
use std::{fs::File, path::Path, sync::Arc};

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(mut settings) = app.world.get_resource_mut::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(settings.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = Self::read_manifest(&settings.game_manifest);
                if settings.assets_to_load.is_empty() {
                    settings.assets_to_load = manifest.resources().copied().collect();
                }

                let mut registry = AssetRegistryOptions::new();
                registry = runtime_data::add_loaders(registry);
                registry = legion_graphics_runtime::add_loaders(registry);
                let registry = registry
                    .add_device_cas(Box::new(content_store), manifest)
                    .create();

                app.insert_resource(registry)
                    .insert_resource(AssetLoadingStates::default())
                    .insert_resource(AssetHandles::default())
                    .insert_resource(AssetToEntityMap::default())
                    .add_startup_system(Self::setup)
                    .add_system(Self::update_registry)
                    .add_system(Self::update_assets);
            } else {
                eprintln!(
                    "Unable to open content storage in {:?}",
                    settings.content_store_addr
                );
            }
        } else {
            eprintln!("Missing AssetRegistrySettings resource, must add to app");
        }
    }
}

impl AssetRegistryPlugin {
    fn setup(
        mut registry: ResMut<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        settings: ResMut<'_, AssetRegistrySettings>,
    ) {
        for asset_id in &settings.assets_to_load {
            asset_loading_states.insert(*asset_id, LoadingState::Pending);
            if let Some(registry) = Arc::get_mut(&mut registry) {
                asset_handles.insert(*asset_id, registry.load_untyped(*asset_id));
            }
        }

        drop(settings);
    }

    fn update_registry(mut registry: ResMut<'_, Arc<AssetRegistry>>) {
        if let Some(registry) = Arc::get_mut(&mut registry) {
            registry.update();
        }
    }

    fn update_assets(
        registry: ResMut<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        asset_handles: ResMut<'_, AssetHandles>,
        mut asset_to_entity_map: ResMut<'_, AssetToEntityMap>,
        mut commands: Commands<'_, '_>,
    ) {
        for (asset_id, loading_state) in asset_loading_states.iter_mut() {
            match loading_state {
                LoadingState::Pending => {
                    let handle = asset_handles.get(*asset_id).unwrap();
                    if handle.is_loaded(&registry) {
                        if !load_ecs_asset::<runtime_data::Entity>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                        ) && !load_ecs_asset::<runtime_data::Instance>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                        ) && !load_ecs_asset::<legion_graphics_runtime::Material>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                        ) && !load_ecs_asset::<runtime_data::Mesh>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                        ) && !load_ecs_asset::<legion_graphics_runtime::Texture>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                        ) {
                            eprintln!(
                                "Unhandled runtime type: {}, asset: {}",
                                asset_id.ty(),
                                asset_id
                            );
                        }

                        *loading_state = LoadingState::Loaded;
                    } else if handle.is_err(&registry) {
                        eprintln!("Failed to load runtime asset {}", asset_id);
                        *loading_state = LoadingState::Failed;
                    }
                }
                LoadingState::Loaded | LoadingState::Failed => {}
            }
        }

        drop(registry);
        drop(asset_handles);
    }

    fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
        match File::open(manifest_path) {
            Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
            Err(_e) => Manifest::default(),
        }
    }
}
