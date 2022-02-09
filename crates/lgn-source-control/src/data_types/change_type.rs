use std::fmt::Display;

use crate::{Error, FileInfo, Result};

/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]

pub enum ChangeType {
    Add {
        new_info: FileInfo,
    },
    Edit {
        old_info: FileInfo,
        new_info: FileInfo,
    },
    Delete {
        old_info: FileInfo,
    },
}

impl ChangeType {
    pub fn new(old_info: Option<FileInfo>, new_info: Option<FileInfo>) -> Option<Self> {
        match (old_info, new_info) {
            (Some(old_info), Some(new_info)) => Some(Self::Edit { old_info, new_info }),
            (Some(old_info), None) => Some(Self::Delete { old_info }),
            (None, Some(new_info)) => Some(Self::Add { new_info }),
            (None, None) => None,
        }
    }

    pub fn old_info(&self) -> Option<&FileInfo> {
        match self {
            ChangeType::Add { .. } => None,
            ChangeType::Edit { old_info, .. } | ChangeType::Delete { old_info } => Some(old_info),
        }
    }

    pub fn new_info(&self) -> Option<&FileInfo> {
        match self {
            ChangeType::Add { new_info } | ChangeType::Edit { new_info, .. } => Some(new_info),
            ChangeType::Delete { .. } => None,
        }
    }

    pub fn to_human_string(&self) -> String {
        match self {
            ChangeType::Add { .. } => "added".to_string(),
            ChangeType::Edit { old_info, new_info } => {
                if old_info != new_info {
                    "modified".to_string()
                } else {
                    "edited".to_string()
                }
            }
            ChangeType::Delete { .. } => "deleted".to_string(),
        }
    }

    pub fn has_modifications(&self) -> bool {
        match self {
            Self::Add { .. } | Self::Delete { .. } => true,
            Self::Edit { old_info, new_info } => old_info != new_info,
        }
    }

    pub fn into_invert(self) -> Self {
        match self {
            Self::Add { new_info } => Self::Delete { old_info: new_info },
            Self::Edit { old_info, new_info } => Self::Edit {
                old_info: new_info,
                new_info: old_info,
            },
            Self::Delete { old_info } => Self::Add { new_info: old_info },
        }
    }
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add { .. } => write!(f, "A"),
            ChangeType::Edit { old_info, new_info } => {
                if old_info != new_info {
                    write!(f, "M")
                } else {
                    write!(f, "C")
                }
            }
            ChangeType::Delete { .. } => write!(f, "D"),
        }
    }
}

impl From<ChangeType> for lgn_source_control_proto::ChangeType {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Add { new_info } => Self {
                old_info: None,
                new_info: Some(new_info.into()),
            },
            ChangeType::Edit { old_info, new_info } => Self {
                old_info: Some(old_info.into()),
                new_info: Some(new_info.into()),
            },
            ChangeType::Delete { old_info } => Self {
                old_info: Some(old_info.into()),
                new_info: None,
            },
        }
    }
}

impl TryFrom<lgn_source_control_proto::ChangeType> for ChangeType {
    type Error = Error;

    fn try_from(change_type: lgn_source_control_proto::ChangeType) -> Result<Self> {
        Ok(if change_type.old_info.is_none() {
            Self::Add {
                new_info: change_type.new_info.ok_or(Error::InvalidChangeType)?.into(),
            }
        } else if change_type.new_info.is_none() {
            Self::Delete {
                old_info: change_type.old_info.ok_or(Error::InvalidChangeType)?.into(),
            }
        } else {
            Self::Edit {
                old_info: change_type.old_info.ok_or(Error::InvalidChangeType)?.into(),
                new_info: change_type.new_info.ok_or(Error::InvalidChangeType)?.into(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fi(hash: &str, size: u64) -> FileInfo {
        FileInfo {
            hash: hash.to_string(),
            size,
        }
    }

    #[test]
    fn test_change_type_new() {
        assert_eq!(
            ChangeType::new(None, Some(fi("new", 123))),
            Some(ChangeType::Add {
                new_info: fi("new", 123),
            }),
        );
        assert_eq!(
            ChangeType::new(Some(fi("old", 123)), Some(fi("new", 123))),
            Some(ChangeType::Edit {
                old_info: fi("old", 123),
                new_info: fi("new", 123),
            }),
        );
        assert_eq!(
            ChangeType::new(Some(fi("old", 123)), None),
            Some(ChangeType::Delete {
                old_info: fi("old", 123),
            }),
        );
        assert_eq!(ChangeType::new(None, None), None,);
    }

    #[test]
    fn test_change_type_old_info() {
        assert_eq!(
            ChangeType::Add {
                new_info: fi("new", 123),
            }
            .old_info(),
            None,
        );
        assert_eq!(
            ChangeType::Edit {
                old_info: fi("old", 123),
                new_info: fi("new", 123),
            }
            .old_info(),
            Some(&fi("old", 123)),
        );
        assert_eq!(
            ChangeType::Delete {
                old_info: fi("old", 123),
            }
            .old_info(),
            Some(&fi("old", 123)),
        );
    }

    #[test]
    fn test_change_type_new_info() {
        assert_eq!(
            ChangeType::Add {
                new_info: fi("new", 123),
            }
            .new_info(),
            Some(&fi("new", 123)),
        );
        assert_eq!(
            ChangeType::Edit {
                old_info: fi("old", 123),
                new_info: fi("new", 123),
            }
            .new_info(),
            Some(&fi("new", 123)),
        );
        assert_eq!(
            ChangeType::Delete {
                old_info: fi("old", 123),
            }
            .new_info(),
            None
        );
    }

    #[test]
    fn test_change_type_from_proto() {
        let proto = lgn_source_control_proto::ChangeType {
            old_info: Some(fi("old", 123).into()),
            new_info: Some(fi("new", 123).into()),
        };

        let change_type = ChangeType::try_from(proto).unwrap();

        assert_eq!(
            change_type,
            ChangeType::Edit {
                old_info: fi("old", 123),
                new_info: fi("new", 123),
            }
        );
    }

    #[test]
    fn test_change_type_into_proto() {
        let change_type = ChangeType::Edit {
            old_info: fi("old", 123),
            new_info: fi("new", 123),
        };

        let proto: lgn_source_control_proto::ChangeType = change_type.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::ChangeType {
                old_info: Some(fi("old", 123).into()),
                new_info: Some(fi("new", 123).into()),
            }
        );
    }

    #[test]
    fn test_change_type_into_invert() {
        assert_eq!(
            ChangeType::Add {
                new_info: fi("new", 123),
            }
            .into_invert(),
            ChangeType::Delete {
                old_info: fi("new", 123),
            }
        );

        assert_eq!(
            ChangeType::Edit {
                old_info: fi("old", 123),
                new_info: fi("new", 123),
            }
            .into_invert(),
            ChangeType::Edit {
                old_info: fi("new", 123),
                new_info: fi("old", 123),
            }
        );

        assert_eq!(
            ChangeType::Delete {
                old_info: fi("old", 123),
            }
            .into_invert(),
            ChangeType::Add {
                new_info: fi("old", 123),
            }
        );
    }
}
