use super::node::Node;

pub struct Graph {
    nodes: Vec<Box<dyn Node>>,
}

impl Graph {
    pub fn new(nodes: Vec<Box<dyn Node>>) -> Self {
        Self { nodes }
    }
}
