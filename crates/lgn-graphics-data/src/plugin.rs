use lgn_app::prelude::*;
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor};
use lgn_ecs::prelude::*;

/// Ecs Graphics Plugin to register type
#[derive(Default)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
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
        crate::offline_psd::PsdFile::register_type(asset_registry);
        crate::offline_png::PngFile::register_type(asset_registry);
        crate::offline_texture::Texture::register_type(asset_registry);
        crate::offline_gltf::GltfFile::register_type(asset_registry);
    }

    #[cfg(feature = "runtime")]
    {
        lgn_data_runtime::ResourceType::register_name(
            crate::runtime_texture::Texture::TYPE,
            crate::runtime_texture::Texture::TYPENAME,
        );
        crate::runtime::register_types(asset_registry);
    }
}
