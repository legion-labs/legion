use crate::{CanonicalPath, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lock {
    pub canonical_path: CanonicalPath,
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

impl From<Lock> for lgn_source_control_proto::Lock {
    fn from(lock: Lock) -> Self {
        Self {
            canonical_path: lock.canonical_path.to_string(),
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        }
    }
}

impl TryFrom<lgn_source_control_proto::Lock> for Lock {
    type Error = Error;

    fn try_from(lock: lgn_source_control_proto::Lock) -> Result<Self> {
        Ok(Self {
            canonical_path: CanonicalPath::new(&lock.canonical_path)?,
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        })
    }
}

impl std::fmt::Display for Lock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.lock_domain_id, self.canonical_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_from_proto() {
        let lock = Lock {
            canonical_path: CanonicalPath::new_from_name("canonical_path"),
            lock_domain_id: "lock_domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
            branch_name: "branch_name".to_string(),
        };

        let proto_lock = lgn_source_control_proto::Lock::from(lock);

        assert_eq!(
            proto_lock,
            lgn_source_control_proto::Lock {
                canonical_path: "/canonical_path".to_string(),
                lock_domain_id: "lock_domain_id".to_string(),
                workspace_id: "workspace_id".to_string(),
                branch_name: "branch_name".to_string(),
            }
        );
    }

    #[test]
    fn test_lock_to_proto() {
        let proto_lock = lgn_source_control_proto::Lock {
            canonical_path: "/canonical_path".to_string(),
            lock_domain_id: "lock_domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
            branch_name: "branch_name".to_string(),
        };

        let lock = Lock::try_from(proto_lock).unwrap();

        assert_eq!(
            lock,
            Lock {
                canonical_path: CanonicalPath::new_from_name("canonical_path"),
                lock_domain_id: "lock_domain_id".to_string(),
                workspace_id: "workspace_id".to_string(),
                branch_name: "branch_name".to_string(),
            }
        );
    }
}
