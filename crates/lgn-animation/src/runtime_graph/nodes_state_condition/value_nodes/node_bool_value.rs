use lgn_input::keyboard::KeyCode;

use crate::runtime_graph::nodes_state_condition::value_nodes::{
    graph_value_type::GraphValueType, node_value::ValueNode,
};

pub trait BoolValueNode: Send + Sync {
    fn verify_time_condition(&mut self, _delta_time: f32) -> bool;

    fn verify_key_event_condition(&mut self, _event: KeyCode) -> bool {
        false
    }
}

impl ValueNode for dyn BoolValueNode {
    #[inline]
    fn get_value_type() -> Option<GraphValueType> {
        Some(GraphValueType::Bool)
    }
}
