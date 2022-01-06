use lgn_app::prelude::*;
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

#[derive(Default)]
pub struct GenericDataPlugin;

impl Plugin for GenericDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(register_resource_types);
        app.add_startup_system(add_loaders);
    }
}

fn register_resource_types(resource_registry: NonSendMut<'_, ResourceRegistryOptions>) {
    #[cfg(feature = "offline")]
    {
        crate::offline::register_resource_types(resource_registry.into_inner());
    }
}

fn add_loaders(asset_registry: NonSendMut<'_, AssetRegistryOptions>) {
    let asset_registry = asset_registry.into_inner();
    #[cfg(feature = "offline")]
    {
        crate::offline::add_loaders(asset_registry);
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::add_loaders(asset_registry);
    }
}
