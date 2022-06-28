use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

#[derive(Default)]
pub struct SampleDataPlugin;

impl Plugin for SampleDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_plugin);
    }
}

#[allow(unused_variables)]
fn init_plugin(asset_registry: NonSendMut<'_, AssetRegistryOptions>) {
    let asset_registry = asset_registry.into_inner();
    register_types(asset_registry);
}

pub fn register_types(asset_registry: &mut AssetRegistryOptions) {
    #[cfg(feature = "offline")]
    {
        crate::offline::register_types(asset_registry);
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::register_types(asset_registry);
    }
}
