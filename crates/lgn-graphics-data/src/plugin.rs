use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

/// Ecs Graphics Plugin to register type
#[derive(Default)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types);

        app.add_startup_system(add_loaders);
    }
}

#[cfg(feature = "offline")]
fn register_resource_types(resource_registry: NonSendMut<'_, ResourceRegistryOptions>) {
    crate::offline::register_resource_types(resource_registry.into_inner())
        .add_type_mut::<crate::offline_psd::PsdFile>()
        .add_type_mut::<crate::offline_png::PngFile>()
        .add_type_mut::<crate::offline_texture::Texture>()
        .add_type_mut::<crate::offline_gltf::GltfFile>();
}

#[allow(unused_variables)]
fn add_loaders(asset_registry: NonSendMut<'_, AssetRegistryOptions>) {
    let asset_registry = asset_registry.into_inner();
    #[cfg(feature = "offline")]
    {
        crate::offline::add_loaders(asset_registry)
            .add_loader_mut::<crate::offline_psd::PsdFile>()
            .add_loader_mut::<crate::offline_png::PngFile>()
            .add_loader_mut::<crate::offline_texture::Texture>()
            .add_loader_mut::<crate::offline_gltf::GltfFile>();
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::add_loaders(asset_registry)
            .add_loader_mut::<crate::runtime_texture::Texture>()
            .add_loader_mut::<crate::runtime::Model>();
    }
}
