use crate::animation_skeleton::Skeleton;
use lgn_math::{Quat, Vec3};
use lgn_transform::components::Transform;

#[component]
pub struct AnimationTransformBundle {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// Necessary intermediate component because a Vec<Vec<>> is not supported by the data code-generation
#[component]
pub struct AnimationTransformBundleVec {
    pub anim_transform_vec: Vec<AnimationTransformBundle>,
}

#[component]
struct AnimationTrack {
    key_frames: Vec<AnimationTransformBundleVec>,
    current_key_frame_index: u32,
    duration_key_frames: Vec<f32>,
    time_since_last_tick: f32,
    looping: bool,

    // Skeleton
    bone_ids: Vec<i32>,
    parent_indices: Vec<i32>,
}

#[component]
pub struct Connection {
    parent_node_id: u32,
    child_node_id: u32,
}

#[component]
pub struct AnimationClipNode {
    id: i32,
    track: AnimationTrack,
}

#[component]
pub struct EditorGraphDefinition2 {
    states: Vec<AnimationClipNode>,
    connections: Vec<Connection>,
}

#[component]
pub trait Node {}

#[component]
pub struct StateMachineNode {}

impl Node for StateMachineNode {}
pub struct EditorGraphDefinition {
    states: Vec<AnimationClipNode>,
    connections: Vec<Connection>,
}
