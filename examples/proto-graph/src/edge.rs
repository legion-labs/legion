use std::sync::Arc;

use crate::{input::Input, output::Output};

pub struct Edge {
    pub from: Arc<Output>,
    pub to: Arc<Input>,
}

impl Edge {}
