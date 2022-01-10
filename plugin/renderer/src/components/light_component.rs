use lgn_ecs::prelude::*;
use lgn_math::Vec3;

pub enum LightType {
    Omnidirectional,
    Directional,
    Spotlight { cone_angle: f32 },
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
