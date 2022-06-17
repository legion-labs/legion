use crate::runtime_graph::node::Node;

#[derive(Clone)]
pub struct TransitionNode {
    pub source_node_id: usize,
    pub target_node_id: usize,
}

impl Node for TransitionNode {}
