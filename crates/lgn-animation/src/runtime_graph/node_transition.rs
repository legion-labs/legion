use crate::runtime_graph::node::Node;

#[derive(Clone)]
pub struct TransitionNode {
    pub(crate) source_node_id: usize,
    pub(crate) target_node_id: usize,
}

impl Node for TransitionNode {}
