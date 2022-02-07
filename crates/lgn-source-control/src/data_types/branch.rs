use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head: String, //commit id
    pub parent: Option<String>,
    pub lock_domain_id: String,
}

impl From<Branch> for lgn_source_control_proto::Branch {
    fn from(branch: Branch) -> Self {
        Self {
            name: branch.name,
            head: branch.head,
            parent: branch.parent.unwrap_or_default(),
            lock_domain_id: branch.lock_domain_id,
        }
    }
}

impl From<lgn_source_control_proto::Branch> for Branch {
    fn from(branch: lgn_source_control_proto::Branch) -> Self {
        Self {
            name: branch.name,
            head: branch.head,
            parent: Some(branch.parent).filter(|s| !s.is_empty()),
            lock_domain_id: branch.lock_domain_id,
        }
    }
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = &self.parent {
            write!(f, "{}@{} (parent: {})", self.name, self.head, parent)
        } else {
            write!(f, "{}@{}", self.name, self.head)
        }
    }
}

impl Branch {
    /// Create a new root branch.
    pub fn new(name: String, head: String) -> Self {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();

        Self {
            name,
            head,
            parent: None,
            lock_domain_id,
        }
    }

    /// Create a new branch that is a children of the current branch.
    pub fn branch_out(&self, name: String) -> Self {
        Self {
            name,
            head: self.head.clone(),
            parent: Some(self.name.clone()),
            lock_domain_id: self.lock_domain_id.clone(),
        }
    }

    // Detach the branch from its parent, generating a new lock domain id.
    pub fn detach(&mut self) {
        assert!(self.parent.is_some());

        self.parent = None;
        self.lock_domain_id = uuid::Uuid::new_v4().to_string();
    }

    /// Attach the branch to the specified parent branch, using its current lock
    /// domain id.
    pub fn attach(&mut self, parent: &Self) {
        assert!(self.parent.is_none());

        self.parent = Some(parent.name.clone());
        self.lock_domain_id = parent.lock_domain_id.clone();
    }

    /// Returns whether the branch is a root branch.
    ///
    /// A root branch has no parent.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_new() {
        let branch = Branch::new("main".to_string(), "abc".to_string());
        assert_eq!(branch.name, "main");
        assert_eq!(branch.head, "abc");
        assert_eq!(branch.parent, None);
        assert!(!branch.lock_domain_id.is_empty());
    }

    #[test]
    fn test_branch_is_root() {
        let branch = Branch::new("main".to_string(), "abc".to_string());
        assert!(branch.is_root());
    }

    #[test]
    fn test_branch_is_not_root() {
        let branch = Branch {
            name: "main".to_string(),
            head: "abc".to_string(),
            parent: Some("parent".to_string()),
            lock_domain_id: "".to_string(),
        };
        assert!(!branch.is_root());
    }

    #[test]
    fn test_branch_display() {
        let branch = Branch {
            name: "main".to_string(),
            head: "abc".to_string(),
            parent: Some("parent".to_string()),
            lock_domain_id: "".to_string(),
        };
        assert_eq!(format!("{}", branch), "main@abc (parent: parent)");
    }

    #[test]
    fn test_branch_to_proto() {
        let branch = Branch {
            name: "main".to_string(),
            head: "abc".to_string(),
            parent: Some("parent".to_string()),
            lock_domain_id: "".to_string(),
        };

        let proto_branch = lgn_source_control_proto::Branch::from(branch);

        assert_eq!(proto_branch.name, "main");
        assert_eq!(proto_branch.head, "abc");
        assert_eq!(proto_branch.parent, "parent");
        assert_eq!(proto_branch.lock_domain_id, "");
    }

    #[test]
    fn test_branch_to_proto_no_parent() {
        let mut branch = Branch::new("main".to_string(), "abc".to_string());
        branch.lock_domain_id = "my_lock_domain".to_string();

        let proto_branch = lgn_source_control_proto::Branch::from(branch);

        assert_eq!(proto_branch.name, "main");
        assert_eq!(proto_branch.head, "abc");
        assert!(proto_branch.parent.is_empty());
        assert_eq!(proto_branch.lock_domain_id, "my_lock_domain");
    }

    #[test]
    fn test_branch_from_proto() {
        let proto_branch = lgn_source_control_proto::Branch {
            name: "main".to_string(),
            head: "abc".to_string(),
            parent: "parent".to_string(),
            lock_domain_id: "".to_string(),
        };

        let branch = Branch::from(proto_branch);

        assert_eq!(branch.name, "main");
        assert_eq!(branch.head, "abc");
        assert_eq!(branch.parent, Some("parent".to_string()));
        assert_eq!(branch.lock_domain_id, "");
    }

    #[test]
    fn test_branch_from_proto_no_parent() {
        let proto_branch = lgn_source_control_proto::Branch {
            name: "main".to_string(),
            head: "abc".to_string(),
            parent: "".to_string(),
            lock_domain_id: "".to_string(),
        };

        let branch = Branch::from(proto_branch);

        assert_eq!(branch.name, "main");
        assert_eq!(branch.head, "abc");
        assert_eq!(branch.parent, None);
        assert_eq!(branch.lock_domain_id, "");
    }
}
