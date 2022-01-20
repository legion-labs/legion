#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lock {
    pub relative_path: String, /* needs to have a stable representation across platforms because
                                * it seeds the hash */
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

impl From<Lock> for lgn_source_control_proto::Lock {
    fn from(lock: Lock) -> Self {
        Self {
            relative_path: lock.relative_path,
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        }
    }
}

impl From<lgn_source_control_proto::Lock> for Lock {
    fn from(lock: lgn_source_control_proto::Lock) -> Self {
        Self {
            relative_path: lock.relative_path,
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_from_proto() {
        let lock = Lock {
            relative_path: "relative_path".to_string(),
            lock_domain_id: "lock_domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
            branch_name: "branch_name".to_string(),
        };

        let proto_lock = lgn_source_control_proto::Lock::from(lock);

        assert_eq!(
            proto_lock,
            lgn_source_control_proto::Lock {
                relative_path: "relative_path".to_string(),
                lock_domain_id: "lock_domain_id".to_string(),
                workspace_id: "workspace_id".to_string(),
                branch_name: "branch_name".to_string(),
            }
        );
    }

    #[test]
    fn test_lock_to_proto() {
        let proto_lock = lgn_source_control_proto::Lock {
            relative_path: "relative_path".to_string(),
            lock_domain_id: "lock_domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
            branch_name: "branch_name".to_string(),
        };

        let lock = Lock::try_from(proto_lock).unwrap();

        assert_eq!(
            lock,
            Lock {
                relative_path: "relative_path".to_string(),
                lock_domain_id: "lock_domain_id".to_string(),
                workspace_id: "workspace_id".to_string(),
                branch_name: "branch_name".to_string(),
            }
        );
    }
}
