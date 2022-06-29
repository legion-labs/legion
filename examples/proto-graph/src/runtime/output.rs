use super::{edge::Edge, types::Type};

#[derive(Default)]
pub struct Output {
    pub value: Type,
    edges: Vec<Edge>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            value: Type::Signal,
            edges: vec![],
        }
    }

    pub fn signal(&self) {
        for edge in &self.edges {}
    }
}
