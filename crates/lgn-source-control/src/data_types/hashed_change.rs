use super::ChangeType;

/// A change to a file in a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedChange {
    pub relative_path: String,
    pub hash: String,
    pub change_type: ChangeType,
}

impl From<HashedChange> for lgn_source_control_proto::HashedChange {
    fn from(hashed_change: HashedChange) -> Self {
        let change_type: lgn_source_control_proto::ChangeType = hashed_change.change_type.into();

        Self {
            relative_path: hashed_change.relative_path,
            hash: hashed_change.hash,
            change_type: change_type as i32,
        }
    }
}

impl TryFrom<lgn_source_control_proto::HashedChange> for HashedChange {
    type Error = anyhow::Error;

    fn try_from(hashed_change: lgn_source_control_proto::HashedChange) -> anyhow::Result<Self> {
        let change_type =
            match lgn_source_control_proto::ChangeType::from_i32(hashed_change.change_type) {
                Some(change_type) => change_type.into(),
                None => {
                    return Err(anyhow::anyhow!(
                        "invalid change type {}",
                        hashed_change.change_type
                    ))
                }
            };

        Ok(Self {
            relative_path: hashed_change.relative_path,
            hash: hashed_change.hash,
            change_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashed_change_from_proto() {
        let proto = lgn_source_control_proto::HashedChange {
            relative_path: "relative_path".to_string(),
            hash: "hash".to_string(),
            change_type: lgn_source_control_proto::ChangeType::Add as i32,
        };

        let hashed_change = HashedChange::try_from(proto).unwrap();

        assert_eq!(
            hashed_change,
            HashedChange {
                relative_path: "relative_path".to_string(),
                hash: "hash".to_string(),
                change_type: ChangeType::Add,
            }
        );
    }

    #[test]
    fn test_hashed_change_into_proto() {
        let hashed_change = HashedChange {
            relative_path: "relative_path".to_string(),
            hash: "hash".to_string(),
            change_type: ChangeType::Add,
        };

        let proto: lgn_source_control_proto::HashedChange = hashed_change.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::HashedChange {
                relative_path: "relative_path".to_string(),
                hash: "hash".to_string(),
                change_type: lgn_source_control_proto::ChangeType::Add as i32,
            }
        );
    }
}
