#[resource()]
pub struct Material {
    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub albedo: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub normal: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub roughness: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub metalness: Option<ResourcePathId>,
}
