use lgn_input::keyboard::KeyCode;

use crate::runtime_graph::nodes_state_condition::value_nodes::node_bool_value::BoolValueNode;

pub struct StateCompletedConditionNode {
    // pub(crate) settings: Settings,
    // pub(crate) source_state_node: StateNode,
    // pub(crate) result: bool,
    // pub(crate) duration_node: FloatValueNode,
    pub(crate) key_events: Vec<KeyCode>,
}

impl BoolValueNode for StateCompletedConditionNode {
    fn verify_time_condition(&mut self, _delta_time: f32) -> bool {
        false
    }

    fn verify_key_event_condition(&mut self, key_event: KeyCode) -> bool {
        for key in &self.key_events {
            if *key == key_event {
                return true;
            }
        }
        false
    }
}

impl StateCompletedConditionNode {
    pub fn initialize_internal() {}
}

pub struct Settings {
    pub(crate) source_state_node_idx: usize,
    pub(crate) transition_duration: f32,
}
