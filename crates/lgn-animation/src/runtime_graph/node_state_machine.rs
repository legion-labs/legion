use crate::runtime_graph::{
    node::Node, node_state::StateNode, node_transition::TransitionNode,
    nodes_state_condition::value_nodes::node_bool_value::BoolValueNode,
};

pub struct StateMachineNode {
    pub(crate) states: Vec<StateInfo>,
    pub(crate) active_state_idx: usize,
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

// Transition
pub struct TransitionInfo {
    pub(crate) transition_node: TransitionNode,
    pub(crate) condition_node: Box<dyn BoolValueNode>,
}

// State
pub struct StateInfo {
    pub(crate) state_node_idx: usize,
    pub(crate) state_node: StateNode,
    pub(crate) transitions: Vec<TransitionInfo>,
}
