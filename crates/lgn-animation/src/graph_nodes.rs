#![allow(dead_code)]
use lgn_transform::components::Transform;

use crate::{
    animation_bone_mask::BoneMask, animation_graph_events::SampledEventRange, animation_pose::Pose,
    animation_sync_track::SyncTrack, components::AnimationClip,
};
pub enum GraphValueType {
    Unknown = 0,
    Bool,
    ID,
    Int,
    Float,
    Vector,
    Target,
    BoneMask,
    Pose,
}

pub trait GraphNode {
    fn instantiate_node(&self) {}
    // fn is_valid(self: &Self);
    // fn is_initialized();
    // fn initialize();

    // #[inline]
    // fn is_node_active() {}

    // #[inline]
    // fn was_updated() {}

    // fn mark_node_active();
    // fn initialize_internal();
    // fn shutdown_internal();
    // const SETTINGS: Settings;
    // const LAST_UPDATE_ID: u32;
    // const INITIALIZATION_COUNT: u32 = 0;
}

pub struct SettingsAnimationClipNode {
    play_in_reverse_value_node_idx: i16,
    sample_root_motion: bool,
    allow_looping: bool,
}

pub struct AnimationClipNode {
    settings: SettingsAnimationClipNode,
    animation: AnimationClip,
    play_in_reverse_value_node: BoolValueNode,
    should_play_in_reverse: bool,
    should_sample_root_motion: bool,
}

impl GraphNode for AnimationClipNode {
    fn instantiate_node(&self) {}
}

impl AnimationClipNode {
    pub fn update() {}

    pub fn initialize_internal() {}
    pub fn shutdown_internal() {}

    pub fn calculate_result() {}
}

pub struct GraphPoseNodeResult {
    task_idx: i8,
    root_motion_delta: Transform,
    sampled_event_range: SampledEventRange,
}

pub struct PoseNode {
    loop_count: i32,
    duration: f32,
    current_time: f32,
    previous_time: f32,
}

impl PoseNode {
    fn initialize() {}

    fn initialize_internal() {}

    fn update() {}

    fn deactivate_branch() {}
}

// impl GraphNode for PoseNode {
//
// }

pub trait ValueNode {
    fn value();
}

// impl GraphNode for dyn ValueNode {
//
// }

pub struct BoolValueNode {}

impl BoolValueNode {
    fn value_type() {}
}

// impl ValueNode for BoolValueNode {
//
// }

pub struct IDValueNode {}

impl IDValueNode {
    fn value_type() {}
}

// impl ValueNode for IDValueNode {
//
// }

pub struct IntValueNode {}

impl IntValueNode {
    fn value_type() {}
}

// impl ValueNode for IntValueNode {
//
// }

pub struct FloatValueNode {}

impl FloatValueNode {
    fn value_type() {}
}

// impl ValueNode for FloatValueNode {
//
// }

pub struct VectorValueNode {}

impl VectorValueNode {
    fn value_type() {}
}

// impl ValueNode for VectorValueNode {
//
// }

pub struct TargetValueNode {}

impl TargetValueNode {
    fn value_type() {}
}

// impl ValueNode for TargetValueNode {
//
// }

pub struct BoneMaskValueNode {}

impl BoneMaskValueNode {
    fn value_type() {}
}

// impl ValueNode for BoneMaskValueNode {
//
// }

pub enum TransitionState {
    None,
    TransitioningIn,
    TransitioningOut,
}

pub struct TimedEvent {
    id: String,
    time_value: f32,
}

pub struct SettingsStateNode {
    child_node_idx: i16,
    entry_events: Vec<String>,
    execute_events: Vec<String>,
    exit_events: Vec<String>,
    timed_remaining_events: Vec<TimedEvent>,
    timed_elapsed_events: Vec<TimedEvent>,
    layer_bone_mask_node_idx: i16,
    is_off_state: bool,
}

pub struct StateNode {
    settings: SettingsStateNode,
    child_node: PoseNode,
    sampled_event_range: SampledEventRange,
    bone_mask_node: BoneMaskValueNode,
    layer_weight_node: FloatValueNode,
    elapsed_time_in_state: f32,
    transition_state: TransitionState,
    is_first_state_update: bool,
}

impl StateNode {
    pub fn update() {}

    pub fn initialize_internal() {}
    pub fn shutdown_internal() {}

    pub fn start_transition_in() {}
    pub fn start_transition_out() {}
    pub fn sample_state_events() {}
    pub fn update_layer_context() {}
}

pub enum SourceType {
    State,
    Transition,
    CachedPose,
}

pub enum TransitionOptions {
    Synchronized,
    ClampDuration,
    KeepSyncEventIndex,
    KeepSyncEventPercentage,
    ForcedTransitionAllowed,
}

pub struct InitializationOptions {
    source_node_result: GraphPoseNodeResult,
    should_cache_pose: bool,
}

pub struct SettingsTransitionNode {
    target_state_node_idx: i16,
    duration_override_node_idx: i16,
    sync_event_offset_override_node_idx: i16,
    // blend_weight_easing_type
    // root_motion_blend: RootMotionBlendMode,
    duration: f32,
    sync_event_offset: f32,
    // transition_options:
}

impl SettingsTransitionNode {
    pub fn instantiate_node() {}
}

// Derives from PoseNode
pub struct TransitionNode {
    settings: SettingsTransitionNode,
    source_node: PoseNode,
    target_node: StateNode,
    duration_override_node: FloatValueNode,
    event_offset_override_node: FloatValueNode,
    bone_mask: BoneMask,
    transition_progress: f32,
    transition_duration: f32,
    sync_event_offset: i32,
    blend_weight: f32,
    sync_track: SyncTrack,
    source_type: SourceType,
}

impl TransitionNode {
    pub fn update() {}

    /* Transition info */
    pub fn start_transition_from_state() {}
    pub fn start_transition_from_transition() {}

    pub fn initialize_internal() {}
    pub fn shutdown_internal() {}

    pub fn initialize_target_state_and_update_transition() {}

    pub fn update_progress() {}
    pub fn update_progress_clamped_synchronized() {}

    pub fn update_layer_context() {}

    pub fn end_source_transition() {}

    pub fn update_unsynchronized() {}
    pub fn update_synchronized() {}

    pub fn update_cached_pose_buffer_id_state() {}
    pub fn transfer_additional_pose_buffer_ids() {}

    pub fn register_pose_tasks_and_update_displacement() {}

    #[inline]
    pub fn calculate_blend_weight() {}
}

// impl PoseNode for TransitionNode {}
