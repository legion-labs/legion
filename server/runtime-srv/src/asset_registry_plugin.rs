use std::path::{Path, PathBuf};

use crate::data_types;
use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetId, AssetRegistry, AssetRegistryOptions, Handle,
};
use legion_ecs::prelude::*;

pub struct AssetRegistrySettings {
    content_store_addr: PathBuf,
    _root_object: String,
}

impl AssetRegistrySettings {
    pub fn new(content_store_addr: impl AsRef<Path>, root_object: &str) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            _root_object: root_object.to_string(),
        }
    }
}

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(settings) = app.world.get_resource::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(settings.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = Manifest::default();

                let asset_options = AssetRegistryOptions::new();
                let asset_options = data_types::register_asset_loaders(asset_options);
                let registry = asset_options.create(Box::new(content_store), manifest);

                app.insert_non_send_resource(registry)
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

        let id = AssetId::new(data_types::ENTITY_TYPE_ID, 0);
        // if let Some(settings) = world.get_resource::<AssetRegistrySettings>() {
        //     let root = settings.root_object;
        // }

        let _root_entity: Handle<data_types::Entity> = registry.load(id);
    }

    fn update(world: &mut World) {
        let world = world.cell();
        let mut registry = world.get_non_send_mut::<AssetRegistry>().unwrap();
        registry.update();
    }
}
