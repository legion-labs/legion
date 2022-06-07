use crate::animation_skeleton::Skeleton;
use lgn_math::{Quat, Vec3};
use lgn_transform::components::Transform;

#[component]
pub struct AnimationTransform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

#[component]
pub struct AnimationTransformBundle {
    pub local: AnimationTransform,
}

#[component]
pub struct VecAnimationTransform {
    pub anim_transform_vec: Vec<AnimationTransformBundle>,
}

#[component]
struct AnimationTrack {
    key_frames: Vec<VecAnimationTransform>,
    current_key_frame_index: i32,
    duration_key_frames: Vec<f32>,
    time_since_last_tick: f32,
    looping: bool,

    /* Skeleton */
    bone_ids: Vec<i32>,
    parent_indices: Vec<i32>,
}

#[component]
pub struct EditorGraphDefinition {
    nodes: Vec<AnimationClipNode>,
    connections: Vec<Connection>,
}

#[component]
pub struct Connection {
    parent_node_id: i32,
    child_node_id: i32,
}

#[component]
pub struct AnimationClipNode {
    id: i32,
    track: AnimationTrack,
}
