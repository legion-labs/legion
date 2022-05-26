#![allow(dead_code)]

use lgn_math::Vec3;

use crate::{
    animation_graph_resources::GraphVariation,
    graph_nodes::{GraphNode, PoseNode},
};

pub struct GraphInstance {
    //graph_variation: GraphVariation,
    nodes: Vec<Box<dyn GraphNode>>,
    //root_node: PoseNode,
}

impl GraphInstance {
    pub(crate) fn new(raw_nodes: &Vec<Vec3>) -> Self {
        // TODO create the nodes from the raw_nodes
        let nodes = Vec::new();
        Self { nodes }
    }

    pub(crate) fn instantiate_node(&self, node: Vec3) {}

    pub(crate) fn create_animation_graph(&self, nodes: &Vec<Vec3>) {}

    pub fn shutdown() {}

    pub fn is_initialized() {}

    pub fn reset() {}

    pub fn update_graph() {}

    #[inline]
    pub fn is_valid_node_index() {}
}
