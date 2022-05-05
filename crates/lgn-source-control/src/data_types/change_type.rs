use std::fmt::Display;

use lgn_content_store::Identifier;

use crate::{Error, MapOtherError, Result};

/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]

pub enum ChangeType {
    Add {
        new_id: Identifier,
    },
    Edit {
        old_id: Identifier,
        new_id: Identifier,
    },
    Delete {
        old_id: Identifier,
    },
}

impl ChangeType {
    pub fn new(old_id: Option<Identifier>, new_id: Option<Identifier>) -> Option<Self> {
        match (old_id, new_id) {
            (Some(old_id), Some(new_id)) => Some(Self::Edit { old_id, new_id }),
            (Some(old_id), None) => Some(Self::Delete { old_id }),
            (None, Some(new_id)) => Some(Self::Add { new_id }),
            (None, None) => None,
        }
    }

    pub fn old_id(&self) -> Option<&Identifier> {
        match self {
            ChangeType::Add { .. } => None,
            ChangeType::Edit { old_id, .. } | ChangeType::Delete { old_id } => Some(old_id),
        }
    }

    pub fn new_id(&self) -> Option<&Identifier> {
        match self {
            ChangeType::Add { new_id } | ChangeType::Edit { new_id, .. } => Some(new_id),
            ChangeType::Delete { .. } => None,
        }
    }

    pub fn to_human_string(&self) -> String {
        match self {
            ChangeType::Add { .. } => "added".to_string(),
            ChangeType::Edit { old_id, new_id } => {
                if old_id != new_id {
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
            Self::Edit { old_id, new_id } => old_id != new_id,
        }
    }

    #[must_use]
    pub fn into_invert(self) -> Self {
        match self {
            Self::Add { new_id } => Self::Delete { old_id: new_id },
            Self::Edit { old_id, new_id } => Self::Edit {
                old_id: new_id,
                new_id: old_id,
            },
            Self::Delete { old_id } => Self::Add { new_id: old_id },
        }
    }
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add { .. } => write!(f, "A"),
            ChangeType::Edit { old_id, new_id } => {
                if old_id != new_id {
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
            ChangeType::Add { new_id } => Self {
                old_id: "".to_string(),
                new_id: new_id.to_string(),
            },
            ChangeType::Edit { old_id, new_id } => Self {
                old_id: old_id.to_string(),
                new_id: new_id.to_string(),
            },
            ChangeType::Delete { old_id } => Self {
                old_id: old_id.to_string(),
                new_id: "".to_string(),
            },
        }
    }
}

impl TryFrom<lgn_source_control_proto::ChangeType> for ChangeType {
    type Error = Error;

    fn try_from(change_type: lgn_source_control_proto::ChangeType) -> Result<Self> {
        Ok(if change_type.old_id.is_empty() {
            Self::Add {
                new_id: change_type
                    .new_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        } else if change_type.new_id.is_empty() {
            Self::Delete {
                old_id: change_type
                    .old_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        } else {
            Self::Edit {
                old_id: change_type
                    .old_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
                new_id: change_type
                    .new_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use lgn_content_store::{Identifier, Provider};

    use super::*;

    fn id(data: &str) -> Identifier {
        Provider::new_in_memory().compute_id(data.as_bytes())
    }

    #[test]
    fn test_change_type_new() {
        assert_eq!(
            ChangeType::new(None, Some(id("new"))),
            Some(ChangeType::Add { new_id: id("new") }),
        );
        assert_eq!(
            ChangeType::new(Some(id("old")), Some(id("new"))),
            Some(ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }),
        );
        assert_eq!(
            ChangeType::new(Some(id("old")), None),
            Some(ChangeType::Delete { old_id: id("old") }),
        );
        assert_eq!(ChangeType::new(None, None), None,);
    }

    #[test]
    fn test_change_type_old_info() {
        assert_eq!(ChangeType::Add { new_id: id("new") }.old_id(), None,);
        assert_eq!(
            ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }
            .old_id(),
            Some(&id("old")),
        );
        assert_eq!(
            ChangeType::Delete { old_id: id("old") }.old_id(),
            Some(&id("old")),
        );
    }

    #[test]
    fn test_change_type_new_info() {
        assert_eq!(
            ChangeType::Add { new_id: id("new") }.new_id(),
            Some(&id("new")),
        );
        assert_eq!(
            ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }
            .new_id(),
            Some(&id("new")),
        );
        assert_eq!(ChangeType::Delete { old_id: id("old") }.new_id(), None);
    }

    #[test]
    fn test_change_type_from_proto() {
        let proto = lgn_source_control_proto::ChangeType {
            old_id: id("old").to_string(),
            new_id: id("new").to_string(),
        };

        let change_type = ChangeType::try_from(proto).unwrap();

        assert_eq!(
            change_type,
            ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }
        );
    }

    #[test]
    fn test_change_type_into_proto() {
        let change_type = ChangeType::Edit {
            old_id: id("old"),
            new_id: id("new"),
        };

        let proto: lgn_source_control_proto::ChangeType = change_type.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::ChangeType {
                old_id: id("old").to_string(),
                new_id: id("new").to_string(),
            }
        );
    }

    #[test]
    fn test_change_type_into_invert() {
        assert_eq!(
            ChangeType::Add { new_id: id("new") }.into_invert(),
            ChangeType::Delete { old_id: id("new") }
        );

        assert_eq!(
            ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }
            .into_invert(),
            ChangeType::Edit {
                old_id: id("new"),
                new_id: id("old"),
            }
        );

        assert_eq!(
            ChangeType::Delete { old_id: id("old") }.into_invert(),
            ChangeType::Add { new_id: id("old") }
        );
    }
}
