use std::fmt::Display;

/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]

pub enum ChangeType {
    Add { new_hash: String },
    Edit { old_hash: String, new_hash: String },
    Delete { old_hash: String },
}

impl ChangeType {
    pub fn new(old_hash: Option<String>, new_hash: Option<String>) -> Option<Self> {
        match (
            old_hash.filter(|s| !s.is_empty()),
            new_hash.filter(|s| !s.is_empty()),
        ) {
            (Some(old_hash), Some(new_hash)) => Some(Self::Edit { old_hash, new_hash }),
            (Some(old_hash), None) => Some(Self::Delete { old_hash }),
            (None, Some(new_hash)) => Some(Self::Add { new_hash }),
            (None, None) => None,
        }
    }

    pub fn old_hash(&self) -> Option<&str> {
        match self {
            ChangeType::Add { .. } => None,
            ChangeType::Edit { old_hash, .. } | ChangeType::Delete { old_hash } => Some(old_hash),
        }
    }

    pub fn new_hash(&self) -> Option<&str> {
        match self {
            ChangeType::Add { new_hash } | ChangeType::Edit { new_hash, .. } => Some(new_hash),
            ChangeType::Delete { .. } => None,
        }
    }
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add { .. } => write!(f, "A"),
            ChangeType::Edit { old_hash, new_hash } => {
                if old_hash != new_hash {
                    write!(f, "M")
                } else {
                    write!(f, "E")
                }
            }
            ChangeType::Delete { .. } => write!(f, "D"),
        }
    }
}

impl From<ChangeType> for lgn_source_control_proto::ChangeType {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Add { new_hash } => Self {
                old_hash: "".to_string(),
                new_hash,
            },
            ChangeType::Edit { old_hash, new_hash } => Self { old_hash, new_hash },
            ChangeType::Delete { old_hash } => Self {
                old_hash,
                new_hash: "".to_string(),
            },
        }
    }
}

impl From<lgn_source_control_proto::ChangeType> for ChangeType {
    fn from(change_type: lgn_source_control_proto::ChangeType) -> Self {
        if change_type.old_hash.is_empty() {
            Self::Add {
                new_hash: change_type.new_hash,
            }
        } else if change_type.new_hash.is_empty() {
            Self::Delete {
                old_hash: change_type.old_hash,
            }
        } else {
            Self::Edit {
                old_hash: change_type.old_hash,
                new_hash: change_type.new_hash,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_new() {
        assert_eq!(
            ChangeType::new(None, Some("new".to_string())),
            Some(ChangeType::Add {
                new_hash: "new".to_string()
            }),
        );
        assert_eq!(
            ChangeType::new(Some("old".to_string()), Some("new".to_string())),
            Some(ChangeType::Edit {
                old_hash: "old".to_string(),
                new_hash: "new".to_string(),
            }),
        );
        assert_eq!(
            ChangeType::new(Some("old".to_string()), None),
            Some(ChangeType::Delete {
                old_hash: "old".to_string(),
            }),
        );
        assert_eq!(
            ChangeType::new(Some("".to_string()), Some("new".to_string())),
            Some(ChangeType::Add {
                new_hash: "new".to_string()
            }),
        );
        assert_eq!(
            ChangeType::new(Some("old".to_string()), Some("".to_string())),
            Some(ChangeType::Delete {
                old_hash: "old".to_string(),
            }),
        );
        assert_eq!(ChangeType::new(None, None), None,);
        assert_eq!(
            ChangeType::new(Some("".to_string()), Some("".to_string())),
            None,
        );
    }

    #[test]
    fn test_change_type_old_hash() {
        assert_eq!(
            ChangeType::Add {
                new_hash: "new".to_string()
            }
            .old_hash(),
            None,
        );
        assert_eq!(
            ChangeType::Edit {
                old_hash: "old".to_string(),
                new_hash: "new".to_string()
            }
            .old_hash(),
            Some("old")
        );
        assert_eq!(
            ChangeType::Delete {
                old_hash: "old".to_string(),
            }
            .old_hash(),
            Some("old")
        );
    }

    #[test]
    fn test_change_type_new_hash() {
        assert_eq!(
            ChangeType::Add {
                new_hash: "new".to_string()
            }
            .new_hash(),
            Some("new"),
        );
        assert_eq!(
            ChangeType::Edit {
                old_hash: "old".to_string(),
                new_hash: "new".to_string()
            }
            .new_hash(),
            Some("new")
        );
        assert_eq!(
            ChangeType::Delete {
                old_hash: "old".to_string()
            }
            .new_hash(),
            None
        );
    }

    #[test]
    fn test_change_type_from_proto() {
        let proto = lgn_source_control_proto::ChangeType {
            old_hash: "old".to_string(),
            new_hash: "new".to_string(),
        };

        let change_type = ChangeType::from(proto);

        assert_eq!(
            change_type,
            ChangeType::Edit {
                old_hash: "old".to_string(),
                new_hash: "new".to_string()
            }
        );
    }

    #[test]
    fn test_change_type_into_proto() {
        let change_type = ChangeType::Edit {
            old_hash: "old".to_string(),
            new_hash: "new".to_string(),
        };

        let proto: lgn_source_control_proto::ChangeType = change_type.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::ChangeType {
                old_hash: "old".to_string(),
                new_hash: "new".to_string(),
            }
        );
    }
}
