use crate::runtime::{Connection, EditorGraphDefinition};
use crate::runtime_graph::node::Node;
use crate::runtime_graph::node_animation_clip::AnimationClipNode;
use crate::runtime_graph::node_state::StateNode;
use crate::runtime_graph::node_state_machine::{StateMachineNode, TransitionInfo};
use crate::runtime_graph::node_transition::TransitionNode;
use crate::runtime_graph::nodes_state_condition::node_time_condition::TimeConditionNode;
use lgn_ecs::component::Component;

use crate::runtime_graph::node_state_machine::StateInfo;

use super::AnimationClip;

#[derive(Component)]
pub struct GraphDefinition {
    pub current_node_index: i32,
    pub nodes: Vec<Box<dyn Node>>,
    pub connections: Vec<Connection>,
}

impl GraphDefinition {
    #[must_use]
    pub fn new(state_machine: &EditorGraphDefinition) -> Self {
        let mut states = Vec::new();
        let mut current_id: u32 = 0;
        let mut transition_nodes = Vec::new();

        for connection in &state_machine.connections {
            transition_nodes.push(TransitionNode {
                source_node_id: connection.parent_node_id,
                target_node_id: connection.child_node_id,
            });
        }

        for state in &state_machine.states {
            let current_clip = AnimationClip::new(&state.track);
            let clip_node = AnimationClipNode {
                id: current_id as i32,
                clip: current_clip,
            };
            let state_node: StateNode = StateNode {
                id: current_id,
                child_node: Box::new(clip_node),
            };
            let transitions = vec![TransitionInfo {
                transition_node: transition_nodes[current_id as usize].clone(),
                condition_node: Box::new(TimeConditionNode {
                    required_elapsed_time: 4.0,
                    time_since_last_verification: 0.0,
                }),
            }];

            states.push(StateInfo {
                state_node_idx: current_id as i16,
                state_node,
                transitions,
            });

            current_id += 1;
        }
        Self {
            current_node_index: 0,
            nodes: vec![Box::new(StateMachineNode {
                states,
                active_state_idx: 0,
            })],
            connections: state_machine.connections.clone(),
        }
    }
}
