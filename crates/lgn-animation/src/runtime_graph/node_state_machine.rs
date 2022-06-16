// Runtime graph node
// Derives from super class Node
// Points to all the states in the state machine
use crate::runtime_graph::{
    node::Node, node_state::StateNode, node_transition::TransitionNode,
    nodes_state_condition::value_nodes::node_bool_value::BoolValueNode,
};

pub struct StateMachineNode {
    // settings: Settings,

    // Transition
    // transition_settings: TransitionSettings,
    // transition_info: TransitionInfo,

    // State
    // state_settings: StateSettings,
    // state_info: StateInfo,

    // StateMachineNode points to all the different states in the state machine through attribute states.
    pub states: Vec<StateInfo>,

    // Current transition node
    // active_transition: *mut TransitionNode,

    // Current state idx
    pub active_state_idx: u32,
}

impl Node for StateMachineNode {
    fn update(&mut self, time: f32) {
        for transition in &mut self.states[self.active_state_idx as usize].transitions {
            if transition.condition_node.verify_condition(time) {
                self.active_state_idx = transition.transition_node.target_node_id;
                break;
            }
        }

        // Update current state child_node
        self.states[self.active_state_idx as usize]
            .state_node
            .child_node
            .update(time);
    }

    fn get_active_state(&mut self) -> Option<&mut StateInfo> {
        Some(&mut self.states[self.active_state_idx as usize])
    }
}

impl StateMachineNode {
    pub fn evaluate_transitions() {}
    pub fn update_transition_stack() {}
}

// Todo! Add derivation from PoseNode::Settings
pub struct Settings {
    state_settings: Vec<StateSettings>,
    // default_state_idx: i16,
}

impl Settings {
    pub fn instantiate_node() {}
}

// Transition
pub struct TransitionInfo {
    pub transition_node: TransitionNode,
    pub condition_node: Box<dyn BoolValueNode>,
    // target_state_idx: i16,
}

pub struct TransitionSettings {
    target_state_idx: i16,
    condition_node_idx: i16,
    transition_node_idx: i16,
}

// State
pub struct StateInfo {
    pub state_node_idx: i16,
    pub state_node: StateNode,
    // entry_condition_node: &BoolValueNode,
    pub transitions: Vec<TransitionInfo>,
}

impl StateInfo {}

pub struct StateSettings {
    state_node_idx: i16,
    entry_condition_node_idx: i16,
    transition_settings: Vec<TransitionSettings>,
}
