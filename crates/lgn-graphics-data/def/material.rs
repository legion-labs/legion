pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

pub enum Filter {
    Nearest,
    Linear,
}

pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

pub struct SamplerData {
    mag_filter: Filter,
    min_filter: Filter,
    mip_filter: Filter,
    wrap_u: WrappingMode,
    wrap_v: WrappingMode,
}

#[resource]
#[derive(Clone)]
pub struct Material {
    #[legion(resource_type = crate::runtime::BinTexture)]
    pub albedo: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime::BinTexture)]
    pub normal: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime::BinTexture)]
    pub roughness: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime::BinTexture)]
    pub metalness: Option<ResourcePathId>,

    #[legion(default=(204, 204, 204))]
    pub base_albedo: Color,

    #[legion(default = 0.0)]
    pub base_metalness: f32,

    #[legion(default = 0.4)]
    pub base_roughness: f32,

    #[legion(default = 0.5)]
    pub reflectance: f32,

    #[legion(default = AlphaMode::Opaque)]
    pub alpha_mode: AlphaMode,

    pub sampler: Option<SamplerData>,
}
