//! The asset registry plugin provides loading of runtime assets.

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

mod asset_entities;
mod asset_handles;
mod asset_to_ecs;
mod config;
mod loading_states;

use std::{fs::File, path::Path, sync::Arc};

use asset_entities::AssetToEntityMap;
use asset_handles::AssetHandles;
use asset_to_ecs::load_ecs_asset;
pub use config::{AssetRegistrySettings, DataBuildConfig};
use lgn_app::prelude::*;
use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryOptions, ResourceLoadEvent,
};
use lgn_ecs::prelude::*;
use lgn_renderer::resources::DefaultMeshes;
use loading_states::{AssetLoadingStates, LoadingState};
use sample_data_runtime as runtime_data;

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut App) {
        if let Some(mut config) = app.world.get_resource_mut::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(config.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = Self::read_or_default(&config.game_manifest);

                if config.assets_to_load.is_empty() {
                    config.assets_to_load = manifest.resources();
                }

                let mut registry_options = AssetRegistryOptions::new();
                // registry = runtime_data::add_loaders(registry);
                // registry = lgn_graphics_runtime::add_loaders(registry);
                // registry = generic_data::runtime::add_loaders(registry);

                if let Some(databuild_config) = &config.databuild_config {
                    registry_options = registry_options.add_device_build(
                        Box::new(content_store),
                        ContentStoreAddr::from(config.content_store_addr.clone()),
                        manifest.clone(),
                        &databuild_config.build_bin,
                        &databuild_config.buildindex,
                        false,
                    );
                } else {
                    registry_options =
                        registry_options.add_device_cas(Box::new(content_store), manifest.clone());
                }

                app.insert_non_send_resource(registry_options)
                    .insert_resource(AssetLoadingStates::default())
                    .insert_resource(AssetHandles::default())
                    .insert_resource(AssetToEntityMap::default())
                    .insert_resource(manifest)
                    .add_startup_system_to_stage(StartupStage::PostStartup, Self::post_setup)
                    .add_system(Self::update_registry)
                    .add_system(Self::update_assets)
                    .add_system(Self::handle_load_events);
            } else {
                eprintln!(
                    "Unable to open content storage in {:?}",
                    config.content_store_addr
                );
            }
        } else {
            eprintln!("Missing AssetRegistrySettings resource, must add to app");
        }
    }
}

impl AssetRegistryPlugin {
    fn post_setup(
        mut commands: Commands<'_, '_>,
        registry: NonSend<'_, AssetRegistryOptions>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        config: ResMut<'_, AssetRegistrySettings>,
    ) {
        let registry = registry.create();

        let load_events = registry.subscribe_to_load_events();
        commands.insert_resource(load_events);

        // Request load for all assets specified in config.
        for asset_id in &config.assets_to_load {
            asset_loading_states.insert(*asset_id, LoadingState::Pending);
            asset_handles.insert(*asset_id, registry.load_untyped(*asset_id));
        }

        commands.insert_resource(registry);
    }

    fn update_registry(registry: ResMut<'_, Arc<AssetRegistry>>) {
        registry.update();

        drop(registry);
    }

    #[allow(clippy::needless_pass_by_value)]
    fn update_assets(
        registry: ResMut<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        asset_handles: ResMut<'_, AssetHandles>,
        mut asset_to_entity_map: ResMut<'_, AssetToEntityMap>,
        mut commands: Commands<'_, '_>,
        default_meshes: Res<'_, DefaultMeshes>,
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
                            &default_meshes,
                        ) && !load_ecs_asset::<runtime_data::Instance>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                            &default_meshes,
                        ) && !load_ecs_asset::<lgn_graphics_runtime::Material>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                            &default_meshes,
                        ) && !load_ecs_asset::<runtime_data::Mesh>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                            &default_meshes,
                        ) && !load_ecs_asset::<lgn_graphics_runtime::Texture>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                            &default_meshes,
                        ) && !load_ecs_asset::<generic_data::runtime::DebugCube>(
                            asset_id,
                            handle,
                            &registry,
                            &mut commands,
                            &mut asset_to_entity_map,
                            &default_meshes,
                        ) {
                            eprintln!(
                                "Unhandled runtime type: {}, asset: {}",
                                asset_id.kind, asset_id
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

    fn handle_load_events(
        load_events_rx: ResMut<'_, crossbeam_channel::Receiver<ResourceLoadEvent>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
    ) {
        while let Ok(event) = load_events_rx.try_recv() {
            match event {
                ResourceLoadEvent::Loaded(asset_handle) => {
                    let asset_id = asset_handle.id();
                    if asset_loading_states.get(asset_id).is_none() {
                        // Received a load event for an untracked asset.
                        // Most likely, this load has occurred because of loading of dependant
                        // resources.
                        asset_loading_states.insert(asset_id, LoadingState::Pending);
                        asset_handles.insert(asset_id, asset_handle);
                    }
                }
                ResourceLoadEvent::LoadError(asset_id, error_kind) => {
                    if asset_loading_states.get(asset_id).is_none() {
                        eprintln!(
                            "Failed to load runtime asset {}, error: {:?}",
                            asset_id, error_kind
                        );
                        asset_loading_states.insert(asset_id, LoadingState::Failed);
                    }
                }
                ResourceLoadEvent::Reloaded(asset_handle) => {
                    let asset_id = asset_handle.id();
                    if asset_loading_states.get(asset_id).is_some() {
                        asset_loading_states.insert(asset_id, LoadingState::Pending);
                    }
                }
                ResourceLoadEvent::Unloaded(_asset_handle) => {
                    // TODO
                }
            }
        }

        drop(load_events_rx);
    }

    fn read_or_default(manifest_path: impl AsRef<Path>) -> Manifest {
        match File::open(manifest_path) {
            Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
            Err(_e) => Manifest::default(),
        }
    }
}
