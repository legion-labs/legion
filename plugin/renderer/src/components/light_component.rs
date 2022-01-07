use lgn_ecs::prelude::*;
use lgn_math::Vec3;

pub enum LightType {
    Omnidirectional,
    Directional { direction: Vec3 },
    Spotlight { direction: Vec3, cone_angle: f32 },
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: (f32, f32, f32),
    pub radiance: f32,
    pub enabled: bool,
    pub picking_id: u32,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Omnidirectional,
            color: (1.0, 1.0, 1.0),
            radiance: 40.0,
            enabled: true,
            picking_id: 0,
        }
    }
}

// GPU components
// TODO: codegen these structs
#[allow(dead_code)]
pub struct OmnidirectionalLight {
    pos: Vec3,
    radiance: f32,
    color: Vec3,
}

impl OmnidirectionalLight {
    pub const SIZE: usize = 32;
}

#[allow(dead_code)]
pub struct DirectionalLight {
    dir: Vec3,
    radiance: f32,
    color: Vec3,
}

impl DirectionalLight {
    pub const SIZE: usize = 32;
}

#[allow(dead_code)]
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
