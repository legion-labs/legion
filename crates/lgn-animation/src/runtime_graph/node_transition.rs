// Todo! Derives from PoseNode
// Todo! Add other elements that are present in KRG class once they are relevant.
use crate::runtime_graph::node::Node;

#[derive(Clone)]
pub struct TransitionNode {
    pub(crate) source_node_id: usize,
    pub(crate) target_node_id: usize,
}

impl Node for TransitionNode {
    fn update(&mut self, time: f32) {}
}

impl TransitionNode {
    pub fn update() {}

    fn initialize_internal() {}
}

// Derives from PoseNode::Settings
#[derive(Clone)]
pub struct Settings {
    // target_state_node_idx: i16,
    // duration: f32,
}

impl Settings {
    pub fn instantiate_node() {}
}

// Eventually add these to TransitionNode if needed!

// pub enum SourceType {
//     State,
//     Transition,
//     CachedPose,
// }

// pub enum TransitionOptions {
//     Synchronized,
//     ClampDuration,
//     KeepSyncEventIndex,
//     KeepSyncEventPercentage,
//     ForcedTransitionAllowed,
// }

// pub struct InitializationOptions {
//     source_node_result: GraphPoseNodeResult,
//     should_cache_pose: bool,
// }
