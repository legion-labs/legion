use lgn_math::prelude::*;

#[component]
struct Transform {
    #[legion(default = Vec3::ZERO)]
    pub position: Vec3,

    #[legion(default = Quat::IDENTITY)]
    pub rotation: Quat,

    #[legion(default = Vec3::ONE)]
    pub scale: Vec3,
}
