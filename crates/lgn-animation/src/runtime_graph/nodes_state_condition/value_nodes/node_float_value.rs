use crate::runtime_graph::nodes_state_condition::value_nodes::{
    graph_value_type::GraphValueType, node_value::ValueNode,
};

pub struct FloatValueNode {}

impl ValueNode for FloatValueNode {
    fn get_value_type() -> GraphValueType {
        GraphValueType::Float
    }
}
