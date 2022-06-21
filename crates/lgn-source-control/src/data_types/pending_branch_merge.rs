use serde::{Deserialize, Serialize};

use crate::{Branch, BranchName, CommitId};

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingBranchMerge {
    pub name: BranchName,
    pub head: CommitId,
}

impl PendingBranchMerge {
    pub fn new(branch: &Branch) -> Self {
        Self {
            name: branch.name.clone(),
            head: branch.head,
        }
    }
}
