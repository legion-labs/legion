use legion_app::Plugin;
use legion_content_store::RamContentStore;
use legion_data_runtime::{manifest::Manifest, AssetRegistry, AssetRegistryOptions};
use legion_ecs::prelude::*;

use crate::data_types;

pub struct AssetRegistryPlugin {}

impl Default for AssetRegistryPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        // construct default AssetRegistry
        let content_store = Box::new(RamContentStore::default());
        let manifest = Manifest::default();
        let asset_options = AssetRegistryOptions::new();
        let asset_options = data_types::register_asset_loaders(asset_options);
        let registry = asset_options.create(content_store, manifest);

        app.insert_non_send_resource(registry)
            .add_system(update_asset_registry.exclusive_system());
    }
}

fn update_asset_registry(world: &mut World) {
    let mut registry = world.get_non_send_resource_mut::<AssetRegistry>().unwrap();
    registry.update();
}
