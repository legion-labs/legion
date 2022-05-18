#![allow(dead_code)]
use lgn_transform::components::Transform;

use crate::animation_graph_events::SampledEventRange;

pub struct Settings {
    node_idx: i16,
}

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
    fn is_valid();
    fn is_initialized();
    fn initialize();

    #[inline]
    fn is_node_active() {
        /* */
    }

    #[inline]
    fn was_updated() {
        /* */
    }

    fn mark_node_active();
    fn initialize_internal();
    fn shutdown_internal();
    const SETTINGS: Settings;
    const LAST_UPDATE_ID: u32;
    const INITIALIZATION_COUNT: u32 = 0;
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
    fn initialize() {
        /* */
    }

    fn initialize_internal() {
        /* */
    }

    fn update() {
        /* */
    }

    fn deactivate_branch() {
        /* */
    }
}

// impl GraphNode for PoseNode {
//     /* */
// }

pub trait ValueNode {
    fn value();
}

// impl GraphNode for dyn ValueNode {
//     /* */
// }

pub struct BoolValueNode {}

impl BoolValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for BoolValueNode {
//     /* */
// }

pub struct IDValueNode {}

impl IDValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for IDValueNode {
//     /* */
// }

pub struct IntValueNode {}

impl IntValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for IntValueNode {
//     /* */
// }

pub struct FloatValueNode {}

impl FloatValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for FloatValueNode {
//     /* */
// }

pub struct VectorValueNode {}

impl VectorValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for VectorValueNode {
//     /* */
// }

pub struct TargetValueNode {}

impl TargetValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for TargetValueNode {
//     /* */
// }

pub struct BoneMaskValueNode {}

impl BoneMaskValueNode {
    fn value_type() {
        /* */
    }
}

// impl ValueNode for BoneMaskValueNode {
//     /* */
// }
