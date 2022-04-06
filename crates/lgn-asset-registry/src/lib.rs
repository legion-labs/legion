//! The asset registry plugin provides loading of runtime assets.

// crate-specific lint exceptions:
//#![allow()]

mod asset_entities;
mod asset_handles;
mod config;
mod errors;
mod events;
mod loading_states;

use std::{path::Path, str::FromStr, sync::Arc};

use lgn_app::prelude::*;
use lgn_async::TokioAsyncRuntime;
use lgn_content_store2::{ChunkIdentifier, Chunker, Config, ContentProvider};
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryEvent, AssetRegistryOptions,
    AssetRegistryScheduling, ResourceLoadEvent,
};
use lgn_ecs::prelude::*;
use lgn_tracing::error;

pub use crate::{
    asset_entities::AssetToEntityMap,
    config::AssetRegistrySettings,
    errors::{Error, Result},
    events::{LoadAssetEvent, LoadManifestEvent},
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
            .add_system(Self::handle_load_manifest_events)
            .add_system(Self::handle_load_asset_events)
            .add_event::<AssetRegistryEvent>()
            .add_event::<LoadManifestEvent>()
            .add_event::<LoadAssetEvent>();
    }
}

impl AssetRegistryPlugin {
    fn pre_setup(world: &mut World) {
        let data_content_provider = Arc::new(
            world
                .resource::<TokioAsyncRuntime>()
                .block_on(async { Config::load_and_instantiate_volatile_provider().await })
                .unwrap(),
        );

        let config = world.resource::<AssetRegistrySettings>();
        let manifest: Option<Manifest> = if let Some(game_manifest) = &config.game_manifest {
            let manifest = {
                let async_rt = world.resource::<TokioAsyncRuntime>();
                let manifest = async_rt.block_on(async {
                    Self::load_manifest_from_path(&game_manifest, &data_content_provider).await
                });
                match manifest {
                    Ok(manifest) => manifest,
                    Err(error) => {
                        error!("error reading manifest: {}", error);
                        Manifest::default()
                    }
                }
            };

            world.insert_resource(manifest.clone());
            Some(manifest)
        } else {
            None
        };

        let mut config = world.resource_mut::<AssetRegistrySettings>();
        if config.assets_to_load.is_empty() {
            if let Some(manifest) = &manifest {
                config.assets_to_load = manifest.resources();
            }
        }

        let mut registry_options = AssetRegistryOptions::new();
        if let Some(manifest) = manifest {
            registry_options = registry_options.add_device_cas(data_content_provider, manifest);
        } else {
            registry_options =
                registry_options.add_device_cas_with_delayed_manifest(data_content_provider);
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

    fn handle_load_manifest_events(
        mut events: EventReader<'_, '_, LoadManifestEvent>,
        registry: Res<'_, Arc<AssetRegistry>>,
    ) {
        for event in events.iter() {
            registry.load_manifest(&event.manifest_id);
        }

        drop(registry);
    }

    fn handle_load_asset_events(
        mut events: EventReader<'_, '_, LoadAssetEvent>,
        registry: Res<'_, Arc<AssetRegistry>>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
    ) {
        for event in events.iter() {
            asset_loading_states.insert(event.asset_id, LoadingState::Pending);
            asset_handles.insert(event.asset_id, registry.load_untyped(event.asset_id));
        }

        drop(registry);
    }

    async fn load_manifest_from_path(
        manifest_path: impl AsRef<Path>,
        content_provider: impl ContentProvider + Send + Sync + Copy,
    ) -> Result<Manifest> {
        let manifest_id = std::fs::read_to_string(manifest_path).map_err(Error::IO)?;
        let manifest_id = ChunkIdentifier::from_str(&manifest_id).map_err(Error::ContentStore)?;
        Self::load_manifest_by_id(manifest_id, content_provider).await
    }

    async fn load_manifest_by_id(
        manifest_id: ChunkIdentifier,
        content_provider: impl ContentProvider + Send + Sync + Copy,
    ) -> Result<Manifest> {
        let chunker = Chunker::default();
        let content = chunker
            .read_chunk(content_provider, &manifest_id)
            .await
            .map_err(Error::ContentStore)?;
        serde_json::from_reader(content.as_slice()).map_err(Error::SerdeJSON)
    }
}
