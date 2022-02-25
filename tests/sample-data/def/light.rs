use lgn_math::prelude::*;

#[component()]
struct Light {
    pub light_type: u32, //TODO: change to enum support when it will be supported
    pub color: Vec3,
    pub radiance: f32,
    pub enabled: bool,
    pub cone_angle: f32,
}
