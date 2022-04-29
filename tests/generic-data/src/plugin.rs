use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

#[derive(Default)]
pub struct GenericDataPlugin;

impl Plugin for GenericDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_loaders);
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
