use std::collections::HashMap;

use super::ScopeDesc;

#[derive(Clone, PartialEq)]
pub struct CallTreeNode {
    pub hash: u32,
    pub begin_ms: f64,
    pub end_ms: f64,
    pub children: Vec<CallTreeNode>,
}

#[derive(Clone, PartialEq)]
pub struct CallTree {
    pub scopes: HashMap<u32, ScopeDesc>,
    pub root: Option<CallTreeNode>,
}
