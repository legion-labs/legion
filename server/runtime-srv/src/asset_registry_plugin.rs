use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_offline::asset::AssetPathId;
use legion_data_runtime::{
    manifest::Manifest, AssetId, AssetRegistry, AssetRegistryOptions, Handle,
};
use legion_ecs::prelude::*;
use sample_data_compiler::runtime_data::{self, CompilableAsset};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct AssetRegistrySettings {
    content_store_addr: PathBuf,
    root_object: String,
}

impl AssetRegistrySettings {
    pub fn new(content_store_addr: impl AsRef<Path>, root_object: &str) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            root_object: root_object.to_string(),
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

                fn add_asset<T>(registry: AssetRegistryOptions) -> AssetRegistryOptions
                where
                    T: CompilableAsset,
                    T::Creator: Send,
                {
                    registry.add_creator(T::TYPE_ID, Box::new(T::Creator::default()))
                }

                let mut registry = AssetRegistryOptions::new();
                registry = add_asset::<runtime_data::Entity>(registry);
                registry = add_asset::<runtime_data::Instance>(registry);
                registry = add_asset::<runtime_data::Material>(registry);
                registry = add_asset::<runtime_data::Mesh>(registry);
                let registry = registry.create(Box::new(content_store), manifest);

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

        if let Some(settings) = world.get_resource::<AssetRegistrySettings>() {
            if let Ok(_asset_path) = AssetPathId::from_str(&settings.root_object) {
                // let id = AssetId::try_from(asset_path.content_type());
                let id = AssetId::new(runtime_data::Entity::TYPE_ID, 0);

                let _root_entity: Handle<runtime_data::Entity> = registry.load(id);
            }
        };
    }

    fn update(world: &mut World) {
        let world = world.cell();
        let mut registry = world.get_non_send_mut::<AssetRegistry>().unwrap();
        registry.update();
    }
}
