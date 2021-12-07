use lgn_ecs::prelude::*;

pub enum LightType {
    Omnidirectional {
        attenuation: f32,
    },
    Directional {
        direction: (f32, f32, f32),
    },
    Spotlight {
        direction: (f32, f32, f32),
        cone_angle: f32,
        attenuation: f32,
    },
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: (f32, f32, f32),
    pub radiance: f32,
}
