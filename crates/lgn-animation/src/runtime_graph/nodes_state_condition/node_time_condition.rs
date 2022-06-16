use crate::runtime_graph::nodes_state_condition::value_nodes::node_bool_value::BoolValueNode;

pub struct TimeConditionNode {
    pub required_elapsed_time: f32,
    pub time_since_last_verification: f32,
}

impl BoolValueNode for TimeConditionNode {
    fn verify_condition(&mut self, delta_time: f32) -> bool {
        self.time_since_last_verification += delta_time;
        if self.time_since_last_verification >= self.required_elapsed_time {
            self.time_since_last_verification -= self.required_elapsed_time;
            return true;
        }
        false
    }
}
