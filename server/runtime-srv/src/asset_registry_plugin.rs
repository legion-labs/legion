use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetId, AssetRegistry, AssetRegistryOptions, HandleUntyped,
};
use legion_ecs::prelude::*;
use sample_data_compiler::runtime_data;
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
    root_assets: Vec<HandleUntyped>,
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
                    .insert_non_send_resource(registry)
                    .add_startup_system(Self::setup.exclusive_system())
                    .add_system(Self::update.exclusive_system());
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
    fn setup(world: &mut World) {
        let world = world.cell();
        let mut registry = world.get_non_send_mut::<AssetRegistry>().unwrap();

        if let Some(settings) = world.get_resource::<AssetRegistrySettings>() {
            if let Ok(asset_id) = AssetId::from_str(&settings.root_asset) {
                let asset = registry.load_untyped(asset_id);

                if let Some(mut state) = world.get_resource_mut::<AssetRegistryState>() {
                    state.root_assets.push(asset);
                }
            }
        };
    }

    fn update(world: &mut World) {
        let world = world.cell();
        let mut registry = world.get_non_send_mut::<AssetRegistry>().unwrap();
        registry.update();
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
