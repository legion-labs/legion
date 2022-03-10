use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::*;
use lgn_graphics_data::{runtime_texture::TextureReferenceType, Color};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask(f32),
    Blend(f32),
}

impl Eq for AlphaMode {}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Component)]
pub struct MaterialComponent {
    pub material_id: ResourceTypeAndId,
    pub albedo_texture: Option<TextureReferenceType>,
    pub base_albedo: Color,
    pub normal_texture: Option<TextureReferenceType>,
    pub metalness_texture: Option<TextureReferenceType>,
    pub base_metalness: f32,
    pub reflectance: f32,
    pub roughness_texture: Option<TextureReferenceType>,
    pub base_roughness: f32,
    pub alpha_mode: AlphaMode,
}

impl MaterialComponent {
    pub fn new(
        material_id: ResourceTypeAndId,
        albedo_texture: Option<TextureReferenceType>,
        normal_texture: Option<TextureReferenceType>,
        metalness_texture: Option<TextureReferenceType>,
        roughness_texture: Option<TextureReferenceType>,
    ) -> Self {
        Self {
            material_id,
            albedo_texture,
            base_albedo: Color::from((204, 204, 204)),
            normal_texture,
            metalness_texture,
            base_metalness: 0.0,
            reflectance: 0.5,
            roughness_texture,
            base_roughness: 0.4,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
