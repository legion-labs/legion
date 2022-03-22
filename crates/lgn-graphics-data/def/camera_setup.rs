use lgn_math::prelude::Vec3;

#[component]
struct CameraSetup {
    eye: Vec3,

    #[legion(default = Vec3::ZERO)]
    look_at: Vec3,
}
