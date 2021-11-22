use legion_ecs::prelude::*;

#[derive(Component)]
pub struct RotationComponent {
    pub rotation_speed: (f32, f32, f32),
}
