use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

#[derive(Default)]
pub struct SampleDataPlugin;

impl Plugin for SampleDataPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types.exclusive_system());

        app.add_startup_system(add_loaders);
    }
}

#[cfg(feature = "offline")]
fn register_resource_types(world: &mut World) {
    if let Some(resource_registry) = world.get_non_send_resource_mut::<ResourceRegistryOptions>() {
        crate::offline::register_resource_types(resource_registry.into_inner());
    }
}

#[allow(unused_variables)]
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
