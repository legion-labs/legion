use crate::runtime_graph::node::Node;

pub struct StateNode {
    pub id: u32,
    pub child_node: Box<dyn Node>,
}

impl Node for StateNode {
    fn update(&mut self, time: f32) {}
}

impl StateNode {}

// Eventually add these to StateNode if needed!

// pub enum TransitionState {
//     None,
//     TransitioningIn,
//     TransitioningOut,
// }
// pub struct Settings {
//     id: u32,
//     child_node: Node,
// }

// impl Settings {
//     pub fn instantiate_node() {}
// }
