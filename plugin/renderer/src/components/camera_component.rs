use lgn_ecs::prelude::*;
use lgn_math::{Quat, Vec3};
use lgn_transform::prelude::*;

#[derive(Component)]
pub struct CameraComponent {
    pub speed: f32,
    pub rotation_speed: f32,
}

impl CameraComponent {
    pub fn default_transform() -> Transform {
        let eye = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(0.0, 0.0, 0.0);

        Transform {
            translation: eye,
            rotation: Quat::from_rotation_arc(
                Vec3::new(0.0, 0.0, -1.0),
                (center - eye).normalize(),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            speed: 2.0,
            rotation_speed: 0.5,
        }
    }
}
