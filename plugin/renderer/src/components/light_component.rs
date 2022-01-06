use lgn_ecs::prelude::*;
use lgn_math::Vec3;

pub enum LightType {
    Omnidirectional,
    Directional { direction: Vec3 },
    Spotlight { direction: Vec3, cone_angle: f32 },
}

pub struct LightSettings {
    pub specular: bool,
    pub diffuse: bool,
    pub specular_reflection: f32,
    pub diffuse_reflection: f32,
    pub ambient_reflection: f32,
    pub shininess: f32,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            specular: true,
            diffuse: true,
            specular_reflection: 1.0,
            diffuse_reflection: 1.0,
            ambient_reflection: 0.2,
            shininess: 16.0,
        }
    }
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: (f32, f32, f32),
    pub radiance: f32,
    pub enabled: bool,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Omnidirectional,
            color: (1.0, 1.0, 1.0),
            radiance: 40.0,
            enabled: true,
        }
    }
}

// GPU components
// TODO: codegen these structs
pub struct OmnidirectionalLight {
    pos: Vec3,
    radiance: f32,
    color: Vec3,
}

impl OmnidirectionalLight {
    pub const SIZE: usize = 32;
}

pub struct DirectionalLight {
    dir: Vec3,
    radiance: f32,
    color: Vec3,
}

impl DirectionalLight {
    pub const SIZE: usize = 32;
}

pub struct Spotlight {
    pos: Vec3,
    radiance: f32,
    dir: Vec3,
    cone_angle: f32,
    color: Vec3,
}

impl Spotlight {
    pub const SIZE: usize = 64;
}
