use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::CommitId;

/// A branch represents a series of commits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head: CommitId,
    pub lock_domain_id: String,
}

impl From<Branch> for lgn_source_control_proto::Branch {
    fn from(branch: Branch) -> Self {
        Self {
            name: branch.name,
            head: branch.head.0,
            lock_domain_id: branch.lock_domain_id,
        }
    }
}

impl From<lgn_source_control_proto::Branch> for Branch {
    fn from(branch: lgn_source_control_proto::Branch) -> Self {
        Self {
            name: branch.name,
            head: CommitId(branch.head),
            lock_domain_id: branch.lock_domain_id,
        }
    }
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.head)
    }
}

impl Branch {
    /// Create a new root branch.
    pub fn new(name: String, head: CommitId) -> Self {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();

        Self {
            name,
            head,
            lock_domain_id,
        }
    }

    /// Advance a branch to a new head.
    pub fn advance(&self, head: CommitId) -> Self {
        Self {
            head,
            name: self.name.clone(),
            lock_domain_id: self.lock_domain_id.clone(),
        }
    }

    /// Create a new branch that points to the same commit and shares the same
    /// lock domain as the current branch.
    pub fn branch_out(&self, name: String) -> Self {
        Self {
            name,
            head: self.head,
            lock_domain_id: self.lock_domain_id.clone(),
        }
    }

    /// Detaches the branch from its siblings, generating a new lock domain id.
    pub fn detach(&mut self) {
        self.lock_domain_id = uuid::Uuid::new_v4().to_string();
    }

    /// Attaches the branch to the specified branch, using its current lock
    /// domain id.
    pub fn attach(&mut self, branch: &Self) {
        self.lock_domain_id = branch.lock_domain_id.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_new() {
        let branch = Branch::new("main".to_string(), CommitId(42));
        assert_eq!(branch.name, "main");
        assert_eq!(branch.head, CommitId(42));
        assert!(!branch.lock_domain_id.is_empty());
    }

    #[test]
    fn test_branch_display() {
        let branch = Branch {
            name: "main".to_string(),
            head: CommitId(42),
            lock_domain_id: "".to_string(),
        };
        assert_eq!(format!("{}", branch), "main@42");
    }

    #[test]
    fn test_branch_to_proto() {
        let branch = Branch {
            name: "main".to_string(),
            head: CommitId(42),
            lock_domain_id: "".to_string(),
        };

        let proto_branch = lgn_source_control_proto::Branch::from(branch);

        assert_eq!(proto_branch.name, "main");
        assert_eq!(proto_branch.head, 42);
        assert_eq!(proto_branch.lock_domain_id, "");
    }

    #[test]
    fn test_branch_from_proto() {
        let proto_branch = lgn_source_control_proto::Branch {
            name: "main".to_string(),
            head: 42,
            lock_domain_id: "".to_string(),
        };

        let branch = Branch::from(proto_branch);

        assert_eq!(branch.name, "main");
        assert_eq!(branch.head, CommitId(42));
        assert_eq!(branch.lock_domain_id, "");
    }
}
