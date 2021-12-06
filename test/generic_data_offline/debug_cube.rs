use lgn_graphics_data::Color;
use lgn_math::prelude::*;

#[data_container()]
struct DebugCube {
    #[legion(default=(0.0,0.0,0.0))]
    pub position: Vec3,

    #[legion(default= Quat::IDENTITY)]
    pub rotation: Quat,

    #[legion(default=(1.0,1.0,1.0))]
    pub scale: Vec3,

    #[legion(default = 1)]
    pub mesh_id: usize,

    #[legion(default=(255,0,0))]
    pub color: Color,

    #[legion(default=(0.0,0.0,0.0))]
    pub rotation_speed: Vec3,
}
