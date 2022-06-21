use crate::{CanonicalPath, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lock {
    pub canonical_path: CanonicalPath,
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

impl From<Lock> for crate::api::source_control::Lock {
    fn from(lock: Lock) -> Self {
        Self {
            canonical_path: lock.canonical_path.into(),
            lock_domain_id: lock.lock_domain_id.into(),
            workspace_id: lock.workspace_id.into(),
            branch_name: lock.branch_name.into(),
        }
    }
}

impl TryFrom<crate::api::source_control::Lock> for Lock {
    type Error = Error;

    fn try_from(lock: crate::api::source_control::Lock) -> Result<Self> {
        Ok(Self {
            canonical_path: lock.canonical_path.try_into()?,
            lock_domain_id: lock.lock_domain_id.0,
            workspace_id: lock.workspace_id.0,
            branch_name: lock.branch_name.0,
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
            canonical_path: CanonicalPath::new("/canonical_path").unwrap(),
            lock_domain_id: "lock_domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
            branch_name: "branch_name".to_string(),
        };

        let proto_lock = crate::api::source_control::Lock::from(lock);

        assert_eq!(
            proto_lock,
            crate::api::source_control::Lock {
                canonical_path: "/canonical_path".to_string().into(),
                lock_domain_id: "lock_domain_id".to_string().into(),
                workspace_id: "workspace_id".to_string().into(),
                branch_name: "branch_name".to_string().into(),
            }
        );
    }

    #[test]
    fn test_lock_to_proto() {
        let proto_lock = crate::api::source_control::Lock {
            canonical_path: "/canonical_path".to_string().into(),
            lock_domain_id: "lock_domain_id".to_string().into(),
            workspace_id: "workspace_id".to_string().into(),
            branch_name: "branch_name".to_string().into(),
        };

        let lock = Lock::try_from(proto_lock).unwrap();

        assert_eq!(
            lock,
            Lock {
                canonical_path: CanonicalPath::new("/canonical_path").unwrap(),
                lock_domain_id: "lock_domain_id".to_string(),
                workspace_id: "workspace_id".to_string(),
                branch_name: "branch_name".to_string(),
            }
        );
    }
}
