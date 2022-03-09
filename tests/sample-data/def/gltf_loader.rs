// TODO: remove when there's a support for dependency handling coming from one asset.
// For now we need to specify at least one model, one material, and one texture to trigger all three gltf compilers
// Instead, materials and textures import should be triggered by importing the models that depend on it
#[component()]
struct GltfLoader {
    #[legion(resource_type = lgn_graphics_data::runtime::Model)]
    pub models: Vec<ResourcePathId>,

    #[legion(resource_type = lgn_graphics_data::runtime::Material)]
    pub materials: Vec<ResourcePathId>,

    #[legion(resource_type = lgn_graphics_data::runtime_texture::Texture)]
    pub textures: Vec<ResourcePathId>,
}
