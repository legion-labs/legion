use crate::runtime_graph::nodes_state_condition::value_nodes::graph_value_type::GraphValueType;

pub trait ValueNode {
    fn get_value_type() -> GraphValueType {
        GraphValueType::None
    }
}
