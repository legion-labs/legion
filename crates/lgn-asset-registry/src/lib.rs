//! The asset registry plugin provides loading of runtime assets.

// crate-specific lint exceptions:
//#![allow()]
mod asset_entities;

mod asset_handles;
mod config;
mod loading_states;

use std::{fs::File, path::Path, sync::Arc};

pub use asset_entities::AssetToEntityMap;
use asset_handles::AssetHandles;
pub use config::{AssetRegistrySettings, DataBuildConfig};
use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_content_store::HddContentStore;
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryEvent, AssetRegistryOptions,
    AssetRegistryScheduling, ResourceLoadEvent,
};
use lgn_ecs::prelude::*;
use lgn_tracing::error;
use loading_states::{AssetLoadingStates, LoadingState};

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetLoadingStates>()
            .init_resource::<AssetHandles>()
            .init_resource::<AssetToEntityMap>()
            .add_startup_system_to_stage(
                StartupStage::PreStartup,
                Self::pre_setup.exclusive_system(),
            )
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                Self::post_setup
                    .exclusive_system()
                    .label(AssetRegistryScheduling::AssetRegistryCreated),
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, Self::preload_assets)
            .add_system(Self::update_registry)
            .add_system(Self::update_assets)
            .add_system(Self::handle_load_events)
            .add_event::<AssetRegistryEvent>();
    }
}

impl AssetRegistryPlugin {
    fn pre_setup(world: &mut World) {
        let mut config = world.resource_mut::<AssetRegistrySettings>();

        let content_store_addr = config.content_store_addr.clone();
        let content_store = HddContentStore::open(content_store_addr).unwrap_or_else(|| {
            panic!(
                "Unable to open content storage in {:?}",
                config.content_store_addr
            )
        });

        let manifest = Self::read_or_default(&config.game_manifest);

        if config.assets_to_load.is_empty() {
            config.assets_to_load = manifest.resources();
        }

        let mut registry_options = AssetRegistryOptions::new();

        if let Some(databuild_config) = &config.databuild_config {
            registry_options = registry_options.add_device_build(
                Box::new(content_store),
                config.content_store_addr.clone(),
                manifest.clone(),
                &databuild_config.build_bin,
                databuild_config.output_db_addr.clone(),
                &databuild_config.project,
                false,
            );
        } else {
            registry_options =
                registry_options.add_device_cas(Box::new(content_store), manifest.clone());
        }

        world.insert_resource(manifest);
        world.insert_non_send_resource(registry_options);
    }

    fn post_setup(world: &mut World) {
        let registry_options = world
            .remove_non_send_resource::<AssetRegistryOptions>()
            .unwrap();

        let async_rt = world.resource::<TokioAsyncRuntime>();
        let registry = async_rt.block_on(async { registry_options.create().await });

        let load_events = registry.subscribe_to_load_events();
        world.insert_resource(load_events);

        world.insert_resource(registry);
    }

    // Request load for all assets specified in config.
    #[allow(clippy::needless_pass_by_value)]
    fn preload_assets(
        config: ResMut<'_, AssetRegistrySettings>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        registry: Res<'_, Arc<AssetRegistry>>,
    ) {
        for asset_id in &config.assets_to_load {
            asset_loading_states.insert(*asset_id, LoadingState::Pending);
            asset_handles.insert(*asset_id, registry.load_untyped(*asset_id));
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn update_registry(registry: Res<'_, Arc<AssetRegistry>>) {
        registry.update();
    }

    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn update_assets(
        registry: Res<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        asset_handles: ResMut<'_, AssetHandles>,
        mut event_writer: EventWriter<'_, '_, AssetRegistryEvent>,
    ) {
        for (asset_id, loading_state) in asset_loading_states.iter_mut() {
            match loading_state {
                LoadingState::Pending => {
                    let handle = asset_handles.get(*asset_id).unwrap();
                    if handle.is_loaded(&registry) {
                        *loading_state = LoadingState::Loaded;
                        event_writer.send(AssetRegistryEvent::AssetLoaded(handle.id()));
                    } else if handle.is_err(&registry) {
                        error!("Failed to load runtime asset {:?}", asset_id);
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
        mut load_events_rx: ResMut<'_, tokio::sync::mpsc::UnboundedReceiver<ResourceLoadEvent>>,
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
                        error!(
                            "Failed to load runtime asset {:?}, error: {:?}",
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
