use lgn_ecs::prelude::*;
use lgn_math::Vec3;

pub enum LightType {
    Omnidirectional {
        attenuation: f32,
    },
    Directional {
        direction: Vec3,
    },
    Spotlight {
        direction: Vec3,
        cone_angle: f32,
        attenuation: f32,
    },
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
            light_type: LightType::Omnidirectional { attenuation: 1.0 },
            color: (1.0, 1.0, 1.0),
            radiance: 40.0,
            enabled: true,
        }
    }
}
