use lgn_math::prelude::*;

#[component]
struct RotationComponent {
    #[legion(default=(0.0,0.0,0.0))]
    pub rotation_speed: Vec3,
}
