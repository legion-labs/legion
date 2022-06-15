//! The asset registry plugin provides loading of runtime assets.

// crate-specific lint exceptions:
//#![allow()]

mod asset_entities;
mod asset_handles;
mod config;
mod errors;
mod events;
mod loading_states;

use std::{path::Path, sync::Arc};

use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_content_store::{
    indexing::{empty_tree_id, ResourceIndex, SharedTreeIdentifier, TreeIdentifier},
    Config,
};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, AssetRegistry, AssetRegistryEvent, AssetRegistryOptions,
    AssetRegistryScheduling, ResourceLoadEvent,
};
use lgn_ecs::prelude::*;
use lgn_tracing::{error, info};

pub use crate::{
    asset_entities::AssetToEntityMap,
    config::AssetRegistrySettings,
    errors::{Error, Result},
    events::AssetRegistryRequest,
};
use crate::{
    asset_handles::AssetHandles,
    loading_states::{AssetLoadingStates, LoadingState},
};

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
            .add_system(Self::handle_requests)
            .add_event::<AssetRegistryEvent>()
            .add_event::<AssetRegistryRequest>();
    }
}

impl AssetRegistryPlugin {
    fn pre_setup(world: &mut World) {
        let data_provider = Arc::new(
            world
                .resource::<TokioAsyncRuntime>()
                .block_on(async { Config::load_and_instantiate_volatile_provider().await })
                .unwrap(),
        );

        let config = world.resource::<AssetRegistrySettings>();
        let mut manifest_id = world.resource::<TokioAsyncRuntime>().block_on(async {
            SharedTreeIdentifier::new(empty_tree_id(&data_provider).await.unwrap())
        });
        if let Some(game_manifest) = &config.game_manifest {
            match Self::load_manifest_id_from_path(&game_manifest) {
                Ok(game_manifest_id) => {
                    manifest_id = SharedTreeIdentifier::new(game_manifest_id);
                }
                Err(error) => {
                    error!("error reading manifest: {}", error);
                }
            }
        };
        let load_all_assets_from_manifest = config.assets_to_load.is_empty();

        world.insert_resource(manifest_id.clone());

        let registry_options = AssetRegistryOptions::new()
            .add_device_cas(Arc::clone(&data_provider), manifest_id.clone());

        if load_all_assets_from_manifest {
            if let Ok(resources) = world.resource::<TokioAsyncRuntime>().block_on(async {
                let manifest = ResourceIndex::new_shared_with_id(
                    new_resource_type_and_id_indexer(),
                    manifest_id.clone(),
                );
                manifest.enumerate_resources(&data_provider).await
            }) {
                let mut config = world.resource_mut::<AssetRegistrySettings>();

                for (index_key, _resource_id) in resources {
                    config.assets_to_load.push(index_key.into());
                }
            }
        }

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
        mut commands: Commands<'_, '_>,
    ) {
        for asset_id in config.assets_to_load.iter().copied() {
            asset_loading_states.insert(asset_id, LoadingState::Pending);
            asset_handles.insert(asset_id, registry.load_untyped(asset_id));
        }

        // Can clean up AssetRegistrySettings, no longer needed
        commands.remove_resource::<AssetRegistrySettings>();
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
                        event_writer.send(AssetRegistryEvent::AssetLoaded(handle.clone()));
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

    fn handle_requests(
        mut events: EventReader<'_, '_, AssetRegistryRequest>,
        registry: Res<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        manifest_id: Res<'_, SharedTreeIdentifier>,
    ) {
        for event in events.iter() {
            match event {
                AssetRegistryRequest::LoadManifest(new_manifest_id) => {
                    info!("received request to load manifest \"{}\"", new_manifest_id);
                    manifest_id.write(new_manifest_id.clone());
                }
                AssetRegistryRequest::LoadAsset(asset_id) => {
                    info!("received request to load asset \"{}\"", asset_id);
                    asset_loading_states.insert(*asset_id, LoadingState::Pending);
                    asset_handles.insert(*asset_id, registry.load_untyped(*asset_id));
                }
            }
        }

        drop(registry);
        drop(manifest_id);
    }

    fn load_manifest_id_from_path(manifest_path: impl AsRef<Path>) -> Result<TreeIdentifier> {
        let manifest_id = std::fs::read_to_string(manifest_path).map_err(Error::IO)?;
        manifest_id
            .parse::<TreeIdentifier>()
            .map_err(Error::ContentStoreIndexing)
    }
}
