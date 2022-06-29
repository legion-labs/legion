use std::sync::Arc;

use super::{edge::Edge, types::Type};

pub struct Input {
    default: Type,
    edge: Option<Arc<Edge>>,
}

impl Input {
    pub fn new(default: Type) -> Self {
        Self {
            default,
            edge: None,
        }
    }

    pub fn get_value(&self) -> Type {
        if let Some(edge_val) = &self.edge {
            edge_val.from.value.clone()
        } else {
            self.default.clone()
        }
    }
}
