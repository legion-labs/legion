use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

/// Ecs Graphics Plugin to register type
#[derive(Default)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_loaders);
    }
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
            .add_loader_mut::<crate::offline_gltf::GltfFile>()
            .add_processor_mut::<crate::offline_psd::PsdFile>()
            .add_processor_mut::<crate::offline_png::PngFile>()
            .add_processor_mut::<crate::offline_texture::Texture>()
            .add_processor_mut::<crate::offline_gltf::GltfFile>();
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::add_loaders(asset_registry)
            .add_loader_mut::<crate::runtime_texture::Texture>()
            .add_loader_mut::<crate::runtime::Model>();
    }
}
