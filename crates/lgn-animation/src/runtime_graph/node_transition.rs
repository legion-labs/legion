use crate::runtime_graph::node::Node;

#[derive(Clone)]
pub struct TransitionNode {
    pub source_node_id: u32,
    pub target_node_id: u32,
}

impl Node for TransitionNode {}
