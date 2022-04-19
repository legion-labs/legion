use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

#[derive(Default)]
pub struct GenericDataPlugin;

impl Plugin for GenericDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(register_types);
    }
}

#[allow(unused_variables)]
fn register_types(asset_registry: NonSendMut<'_, AssetRegistryOptions>) {
    let asset_registry = asset_registry.into_inner();
    #[cfg(feature = "offline")]
    {
        crate::offline::register_types(asset_registry);
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::register_types(asset_registry);
    }
}
