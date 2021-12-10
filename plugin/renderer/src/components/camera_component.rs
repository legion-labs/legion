use lgn_ecs::prelude::*;
use lgn_math::Vec3;

#[derive(Component)]
pub struct CameraComponent {
    pub pos: Vec3,
    pub up: Vec3,
    pub dir: Vec3,
    pub speed: f32,
    pub rotation_speed: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        let pos = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        Self {
            pos,
            up,
            dir: center - pos,
            speed: 0.5,
            rotation_speed: 0.5,
        }
    }
}
