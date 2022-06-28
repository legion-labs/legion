use crate::runtime_graph::{
    node_state::StateNode,
    nodes_state_condition::value_nodes::{
        node_bool_value::BoolValueNode, node_float_value::FloatValueNode,
    },
};

pub struct StateCompletedConditionNode {
    pub(crate) settings: Settings,

    pub(crate) source_state_node: StateNode,
    pub(crate) result: bool,
    pub(crate) duration_node: FloatValueNode,
}

impl BoolValueNode for StateCompletedConditionNode {
    fn verify_condition(&mut self, _delta_time: f32) -> bool {
        false
    }
}

impl StateCompletedConditionNode {
    pub fn initialize_internal() {}
}

pub struct Settings {
    pub(crate) source_state_node_idx: usize,
    pub(crate) transition_duration: f32,
}
