use crate::animation_skeleton::Skeleton;
use lgn_math::{Quat, Vec3};
use lgn_transform::components::Transform;

#[component]
pub struct AnimationTransformBundle {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

/* Necessary intermediate component because a Vec<Vec<>> is not serializable */
#[component]
pub struct AnimationTransformBundleVec {
    pub anim_transform_vec: Vec<AnimationTransformBundle>,
}

#[component]
struct AnimationTrack {
    key_frames: Vec<AnimationTransformBundleVec>,
    current_key_frame_index: i32,
    duration_key_frames: Vec<f32>,
    time_since_last_tick: f32,
    looping: bool,

    /* Skeleton */
    bone_ids: Vec<i32>,
    parent_indices: Vec<i32>,
}
