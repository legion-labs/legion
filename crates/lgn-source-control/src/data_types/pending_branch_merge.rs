use serde::{Deserialize, Serialize};

use crate::Branch;

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingBranchMerge {
    pub name: String,
    pub head: String, //commit id
}

impl PendingBranchMerge {
    pub fn new(branch: &Branch) -> Self {
        Self {
            name: branch.name.clone(),
            head: branch.head.clone(),
        }
    }
}
