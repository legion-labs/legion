use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{BranchName, CommitId, Error, Result};

/// A branch represents a series of commits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Branch {
    pub name: BranchName,
    pub head: CommitId,
    pub lock_domain_id: String,
}

/// `NewBranch` represents a branch to create.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NewBranch {
    pub name: BranchName,
    pub head: CommitId,
    pub lock_domain_id: String,
}

/// `UpdateBranch` represents a branch update.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UpdateBranch {
    pub head: CommitId,
    pub lock_domain_id: String,
}

impl From<Branch> for crate::api::source_control::Branch {
    fn from(branch: Branch) -> Self {
        Self {
            name: branch.name.into(),
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.into(),
        }
    }
}

impl TryFrom<crate::api::source_control::Branch> for Branch {
    type Error = Error;

    fn try_from(branch: crate::api::source_control::Branch) -> Result<Self> {
        Ok(Self {
            name: branch.name.try_into()?,
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.0,
        })
    }
}

impl From<NewBranch> for crate::api::source_control::NewBranch {
    fn from(branch: NewBranch) -> Self {
        Self {
            name: branch.name.into(),
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.into(),
        }
    }
}

impl TryFrom<crate::api::source_control::NewBranch> for NewBranch {
    type Error = Error;

    fn try_from(branch: crate::api::source_control::NewBranch) -> Result<Self> {
        Ok(Self {
            name: branch.name.try_into()?,
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.0,
        })
    }
}

impl From<UpdateBranch> for crate::api::source_control::UpdateBranch {
    fn from(branch: UpdateBranch) -> Self {
        Self {
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.into(),
        }
    }
}

impl From<crate::api::source_control::UpdateBranch> for UpdateBranch {
    fn from(branch: crate::api::source_control::UpdateBranch) -> Self {
        Self {
            head: branch.head.into(),
            lock_domain_id: branch.lock_domain_id.0,
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
    pub fn new(name: BranchName, head: CommitId) -> Self {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();

        Self {
            name,
            head,
            lock_domain_id,
        }
    }

    /// Advance a branch to a new head.
    #[must_use]
    pub fn advance(self, head: CommitId) -> UpdateBranch {
        UpdateBranch {
            head,
            lock_domain_id: self.lock_domain_id,
        }
    }

    /// Create a new branch that points to the same commit and shares the same
    /// lock domain as the current branch.
    #[must_use]
    pub fn branch_out(self, name: BranchName) -> NewBranch {
        NewBranch {
            name,
            head: self.head,
            lock_domain_id: self.lock_domain_id,
        }
    }

    /// Detaches the branch from its siblings, generating a new lock domain id.
    pub fn detach(self) -> NewBranch {
        NewBranch {
            name: self.name,
            head: self.head,
            lock_domain_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Attaches the branch to the specified branch, using its current lock
    /// domain id.
    pub fn attach(self, branch: &Self) -> NewBranch {
        NewBranch {
            name: self.name,
            head: self.head,
            lock_domain_id: branch.lock_domain_id.clone(),
        }
    }
}

impl NewBranch {
    /// Create a new branch.
    pub fn new(name: BranchName, head: CommitId) -> Self {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();

        Self {
            name,
            head,
            lock_domain_id,
        }
    }

    /// Converts a new branch into a branch.
    pub fn into_branch(self) -> Branch {
        Branch {
            name: self.name,
            head: self.head,
            lock_domain_id: self.lock_domain_id,
        }
    }
}

impl UpdateBranch {
    /// Converts a branch update into a branch.
    pub fn into_branch(self, name: BranchName) -> Branch {
        Branch {
            name,
            head: self.head,
            lock_domain_id: self.lock_domain_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_new() {
        let branch = Branch::new("main".parse().unwrap(), CommitId(42));
        assert_eq!(branch.name, "main".parse().unwrap());
        assert_eq!(branch.head, CommitId(42));
        assert!(!branch.lock_domain_id.is_empty());
    }

    #[test]
    fn test_branch_display() {
        let branch = Branch {
            name: "main".parse().unwrap(),
            head: CommitId(42),
            lock_domain_id: "".to_string(),
        };
        assert_eq!(format!("{}", branch), "main@42");
    }

    #[test]
    fn test_branch_to_api() {
        let branch = Branch {
            name: "main".parse().unwrap(),
            head: CommitId(42),
            lock_domain_id: "".to_string(),
        };

        let api_branch = crate::api::source_control::Branch::from(branch);

        assert_eq!(api_branch.name, "main".to_string().into());
        assert_eq!(api_branch.head, 42.into());
        assert_eq!(api_branch.lock_domain_id, "".to_string().into());
    }

    #[test]
    fn test_branch_from_api() {
        let api_branch = crate::api::source_control::Branch {
            name: "main".to_string().into(),
            head: 42.into(),
            lock_domain_id: "".to_string().into(),
        };

        let branch = Branch::try_from(api_branch).unwrap();

        assert_eq!(branch.name, "main".parse().unwrap());
        assert_eq!(branch.head, CommitId(42));
        assert_eq!(branch.lock_domain_id, "");
    }
}
