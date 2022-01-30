use lgn_math::prelude::*;

#[component]
struct Transform {
    #[legion(default=(0.0,0.0,0.0))]
    pub position: Vec3,

    #[legion(default= Quat::IDENTITY)]
    pub rotation: Quat,

    #[legion(default=(1.0,1.0,1.0))]
    pub scale: Vec3,

    pub apply_to_children: bool,
}
