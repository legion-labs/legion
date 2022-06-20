use crate::runtime_graph::{
    node::Node, node_state::StateNode, node_transition::TransitionNode,
    nodes_state_condition::value_nodes::node_bool_value::BoolValueNode,
};

pub struct StateMachineNode {
    pub(crate) states: Vec<StateInfo>,
    pub(crate) active_state_idx: usize,
    // settings: Settings,

    // Transition
    // transition_settings: TransitionSettings,
    // transition_info: TransitionInfo,

    // State
    // state_settings: StateSettings,
    // state_info: StateInfo,

    // Current transition node
    // active_transition: *mut TransitionNode,
}

impl Node for StateMachineNode {
    fn update(&mut self, time: f32) {
        for transition in &mut self.states[self.active_state_idx].transitions {
            if transition.condition_node.verify_condition(time) {
                self.active_state_idx = transition.transition_node.target_node_id;
                break;
            }
        }

        // Update current state's child_node
        self.states[self.active_state_idx]
            .state_node
            .child_node
            .update(time);
    }

    fn get_active_state(&self) -> Option<&StateInfo> {
        Some(&self.states[self.active_state_idx])
    }
}

impl StateMachineNode {
    pub fn evaluate_transitions() {}
    pub fn update_transition_stack() {}
}

// Todo! Add derivation from PoseNode::Settings
pub struct Settings {
    // state_settings: Vec<StateSettings>,
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
    pub(crate) state_node_idx: usize,
    pub(crate) state_node: StateNode,
    pub(crate) transitions: Vec<TransitionInfo>,
}
