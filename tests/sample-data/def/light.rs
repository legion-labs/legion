use lgn_graphics_data::Color;
use lgn_math::prelude::*;

enum LightType {
    Omnidirectional,
    Directional,
    Spotlight,
}

#[component()]
struct Light {
    pub light_type: LightType,
    #[legion(default=(255,255,255))]
    pub color: Color,
    pub radiance: f32,
    pub enabled: bool,
    pub cone_angle: f32,
}
