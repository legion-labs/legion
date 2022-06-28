use crate::components::animation_clip::AnimationClip;
use crate::runtime::{Connection, EditorGraphDefinition};
use crate::runtime_graph::node::Node;
use crate::runtime_graph::node_animation_clip::AnimationClipNode;
use crate::runtime_graph::node_state::StateNode;
use crate::runtime_graph::node_state_machine::StateInfo;
use crate::runtime_graph::node_state_machine::{StateMachineNode, TransitionInfo};
use crate::runtime_graph::node_transition::TransitionNode;
use crate::runtime_graph::nodes_state_condition::node_time_condition::TimeConditionNode;
use lgn_ecs::component::Component;

#[derive(Component)]
pub struct GraphDefinition {
    pub(crate) current_node_index: usize,
    pub(crate) nodes: Vec<Box<dyn Node>>,
    pub(crate) connections: Vec<Connection>,
}

impl GraphDefinition {
    #[must_use]
    pub fn new(state_machine: &EditorGraphDefinition) -> Self {
        let mut states = Vec::new();
        let mut current_id: usize = 0;
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
                id: current_id,
                clip: current_clip,
            };
            let state_node: StateNode = StateNode {
                id: current_id,
                child_node: Box::new(clip_node),
            };
            let transitions = vec![TransitionInfo {
                transition_node: transition_nodes[current_id].clone(),
                condition_node: Box::new(TimeConditionNode {
                    required_elapsed_time: 4.0,
                    time_since_last_verification: 0.0,
                }),
            }];

            states.push(StateInfo {
                state_node_idx: current_id,
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

    pub(crate) fn update(&mut self, delta_time: f32) {
        self.get_current_node_mut().update(delta_time);
    }

    #[allow(clippy::borrowed_box)]
    pub(crate) fn get_current_node(&self) -> &Box<dyn Node> {
        &self.nodes[self.current_node_index]
    }

    pub(crate) fn get_current_node_mut(&mut self) -> &mut Box<dyn Node> {
        &mut self.nodes[self.current_node_index]
    }
}
