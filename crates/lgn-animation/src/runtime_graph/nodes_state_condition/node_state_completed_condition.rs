use core::time;

use lgn_core::Time;

use crate::runtime_graph::{
    node_state::StateNode,
    nodes_state_condition::value_nodes::{
        node_bool_value::BoolValueNode, node_float_value::FloatValueNode,
    },
};

pub struct StateCompletedConditionNode {
    settings: Settings,

    source_state_node: StateNode,
    result: bool,
    duration_node: FloatValueNode,
}

impl BoolValueNode for StateCompletedConditionNode {
    fn verify_condition(&mut self, delta_time: f32) -> bool {
        // Todo!
        false
    }
}

impl StateCompletedConditionNode {
    pub fn initialize_internal() {}
}

pub struct Settings {
    source_state_node_idx: i16,
    transition_duration: f32,
}
