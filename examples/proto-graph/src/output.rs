use crate::{edge::Edge, types::Type};

pub struct Output {
    pub value: Type,
    edges: Vec<Edge>,
}

impl Output {
    pub fn signal(&self) {
        for edge in &self.edges {}
    }
}
