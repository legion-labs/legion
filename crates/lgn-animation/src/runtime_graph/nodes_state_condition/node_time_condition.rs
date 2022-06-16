use crate::runtime_graph::nodes_state_condition::value_nodes::node_bool_value::BoolValueNode;

pub struct TimeConditionNode {
    pub(crate) required_elapsed_time: f32,
    pub(crate) time_since_last_verification: f32,
    // settings: Settings,
    // source_state_node: *const StateNode,
    // input_value_node: *const FloatValueNode,
    // pub result: bool,
}

impl BoolValueNode for TimeConditionNode {
    fn verify_condition(&mut self, delta_time: f32) -> bool {
        self.time_since_last_verification += delta_time;
        if self.time_since_last_verification >= self.required_elapsed_time {
            self.time_since_last_verification -= self.required_elapsed_time;
            return true;
        }
        false
    }
}

impl TimeConditionNode {
    pub fn initialize_internal() {}
}

pub struct Settings {
    // source_state_node_idx: i16,
    // input_value_node_idx: i16,
    // comparer: f32,
}

impl Settings {
    pub fn instantiate_node() {}
}

// Eventually add these to TimeConditionNode if needed!
// enum ComparisonType {
//     PercentageThroughState,
//     PercentageThroughSyncEvent,
//     LoopCount,
//     ElapsedTime,
// }
// enum Operator {
//     LessThan = 0,
//     LessThanEqual,
//     GreaterThan,
//     GreaterThanEqual,
// }
