use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetId, AssetRegistry, AssetRegistryOptions, HandleUntyped,
};
use legion_ecs::prelude::*;
use sample_data_compiler::runtime_data::{self, CompilableAsset};
use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct AssetRegistrySettings {
    content_store_addr: PathBuf,
    game_manifest: PathBuf,
    root_asset: String,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        root_asset: &str,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            root_asset: root_asset.to_string(),
        }
    }
}

#[derive(Default)]
struct AssetRegistryState {
    assets: Vec<(HandleUntyped, AssetState)>,
}

#[derive(PartialEq)]
enum AssetState {
    PendingLoad,
    Loaded,
}

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(settings) = app.world.get_resource::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(settings.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = read_manifest(&settings.game_manifest);

                let mut registry = AssetRegistryOptions::new();
                registry = runtime_data::add_creators(registry);
                let registry = registry.create(Box::new(content_store), manifest);

                app.insert_resource(AssetRegistryState::default())
                    .insert_resource(registry)
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
        mut state: ResMut<'_, AssetRegistryState>,
        mut registry: ResMut<'_, AssetRegistry>,
        settings: ResMut<'_, AssetRegistrySettings>,
    ) {
        if let Ok(asset_id) = AssetId::from_str(&settings.root_asset) {
            let asset = registry.load_untyped(asset_id);
            state.assets.push((asset, AssetState::PendingLoad));
        }

        drop(settings);
    }

    fn update_registry(mut registry: ResMut<'_, AssetRegistry>) {
        registry.update();
    }

    fn update_assets(
        mut commands: Commands<'_>,
        mut state: ResMut<'_, AssetRegistryState>,
        registry: ResMut<'_, AssetRegistry>,
    ) {
        state
            .assets
            .iter_mut()
            .for_each(|(asset, state)| match *state {
                AssetState::PendingLoad => {
                    if asset.is_loaded(&registry) {
                        if let Some(asset_id) = asset.get_asset_id(&registry) {
                            match asset_id.asset_type() {
                                runtime_data::Entity::TYPE_ID => {
                                    if let Some(entity) =
                                        asset.get::<runtime_data::Entity>(&registry)
                                    {
                                        Self::on_loaded_entity(&mut commands, entity);
                                    }
                                }
                                runtime_data::Instance::TYPE_ID => {
                                    if let Some(instance) =
                                        asset.get::<runtime_data::Instance>(&registry)
                                    {
                                        Self::on_loaded_instance(&mut commands, instance);
                                    }
                                }
                                _ => {}
                            }
                        }
                        *state = AssetState::Loaded;
                    }
                }
                AssetState::Loaded => {}
            });

        drop(registry);
    }

    fn on_loaded_entity(_commands: &mut Commands<'_>, entity: &runtime_data::Entity) {
        println!("Loaded entity {}", entity.name);
        //commands.spawn();
    }

    fn on_loaded_instance(_commands: &mut Commands<'_>, _instance: &runtime_data::Instance) {
        println!("Loaded instance");
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
