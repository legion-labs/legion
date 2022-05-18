pub enum MagFilter {
    Nearest,
    Linear,
}

pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

pub struct Sampler {
    mag_filter: MagFilter,
    min_filter: MinFilter,
    wrap_u: WrappingMode,
    wrap_v: WrappingMode,
}

#[resource()]
#[derive(Clone)]
pub struct Material {
    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub albedo: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub normal: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub roughness: Option<ResourcePathId>,

    #[legion(resource_type = crate::runtime_texture::Texture)]
    pub metalness: Option<ResourcePathId>,

    #[legion(default=(255,0,0))]
    pub base_albedo: Color,

    #[legion(default = 0.0)]
    pub base_metalness: f32,

    #[legion(default = 0.0)]
    pub base_roughness: f32,

    #[legion(default = 0.0)]
    pub reflectance: f32,

    pub sampler: Option<Sampler>,
}
