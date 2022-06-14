use crate::runtime::{Connection, EditorGraphDefinition};
use crate::tmp::node_animation_clip::AnimationClipNode;
use lgn_ecs::component::Component;

use super::AnimationClip;

#[derive(Component, Clone)]
pub struct GraphDefinition {
    pub current_node_index: i32,
    pub nodes: Vec<AnimationClipNode>,
    pub connections: Vec<Connection>,
}

impl GraphDefinition {
    #[must_use]
    pub fn new(raw_anim_graph: &EditorGraphDefinition) -> Self {
        let mut runtime_nodes: Vec<AnimationClipNode> = Vec::new();
        let current_id = 1;
        for node in &raw_anim_graph.nodes {
            let current_clip = AnimationClip::new(&node.track);
            runtime_nodes.push(AnimationClipNode {
                id: current_id,
                clip: current_clip,
            });
        }
        Self {
            current_node_index: 0,
            nodes: runtime_nodes,
            connections: raw_anim_graph.connections.clone(),
        }
    }
}
