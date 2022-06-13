use crate::{
    animation_graph_resources::GraphVariation,
    graph_nodes::{GraphNode, GraphPoseNodeResult, PoseNode},
};

pub struct GraphInstance {
    graph_variation: GraphVariation,
    // nodes: Vec<GraphNode>,
    root_node: PoseNode,
}

impl GraphInstance {
    pub fn initialize() {
        /* */
    }

    pub fn shutdown() {
        /* */
    }

    pub fn is_initialized() {
        /* */
    }

    pub fn reset() {
        /* */
    }

    pub fn update_graph() {
        /* */
    }

    #[inline]
    pub fn is_valid_node_index() {
        /* */
    }
}
