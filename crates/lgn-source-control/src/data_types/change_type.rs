use std::fmt::Display;

use lgn_content_store2::ChunkIdentifier;

use crate::{Error, MapOtherError, Result};

/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]

pub enum ChangeType {
    Add {
        new_chunk_id: ChunkIdentifier,
    },
    Edit {
        old_chunk_id: ChunkIdentifier,
        new_chunk_id: ChunkIdentifier,
    },
    Delete {
        old_chunk_id: ChunkIdentifier,
    },
}

impl ChangeType {
    pub fn new(
        old_chunk_id: Option<ChunkIdentifier>,
        new_chunk_id: Option<ChunkIdentifier>,
    ) -> Option<Self> {
        match (old_chunk_id, new_chunk_id) {
            (Some(old_chunk_id), Some(new_chunk_id)) => Some(Self::Edit {
                old_chunk_id,
                new_chunk_id,
            }),
            (Some(old_chunk_id), None) => Some(Self::Delete { old_chunk_id }),
            (None, Some(new_chunk_id)) => Some(Self::Add { new_chunk_id }),
            (None, None) => None,
        }
    }

    pub fn old_chunk_id(&self) -> Option<&ChunkIdentifier> {
        match self {
            ChangeType::Add { .. } => None,
            ChangeType::Edit { old_chunk_id, .. } | ChangeType::Delete { old_chunk_id } => {
                Some(old_chunk_id)
            }
        }
    }

    pub fn new_chunk_id(&self) -> Option<&ChunkIdentifier> {
        match self {
            ChangeType::Add { new_chunk_id } | ChangeType::Edit { new_chunk_id, .. } => {
                Some(new_chunk_id)
            }
            ChangeType::Delete { .. } => None,
        }
    }

    pub fn to_human_string(&self) -> String {
        match self {
            ChangeType::Add { .. } => "added".to_string(),
            ChangeType::Edit {
                old_chunk_id,
                new_chunk_id,
            } => {
                if old_chunk_id != new_chunk_id {
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
            Self::Edit {
                old_chunk_id,
                new_chunk_id,
            } => old_chunk_id != new_chunk_id,
        }
    }

    pub fn into_invert(self) -> Self {
        match self {
            Self::Add { new_chunk_id } => Self::Delete {
                old_chunk_id: new_chunk_id,
            },
            Self::Edit {
                old_chunk_id,
                new_chunk_id,
            } => Self::Edit {
                old_chunk_id: new_chunk_id,
                new_chunk_id: old_chunk_id,
            },
            Self::Delete { old_chunk_id } => Self::Add {
                new_chunk_id: old_chunk_id,
            },
        }
    }
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add { .. } => write!(f, "A"),
            ChangeType::Edit {
                old_chunk_id,
                new_chunk_id,
            } => {
                if old_chunk_id != new_chunk_id {
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
            ChangeType::Add { new_chunk_id } => Self {
                old_chunk_id: "".to_string(),
                new_chunk_id: new_chunk_id.to_string(),
            },
            ChangeType::Edit {
                old_chunk_id,
                new_chunk_id,
            } => Self {
                old_chunk_id: old_chunk_id.to_string(),
                new_chunk_id: new_chunk_id.to_string(),
            },
            ChangeType::Delete { old_chunk_id } => Self {
                old_chunk_id: old_chunk_id.to_string(),
                new_chunk_id: "".to_string(),
            },
        }
    }
}

impl TryFrom<lgn_source_control_proto::ChangeType> for ChangeType {
    type Error = Error;

    fn try_from(change_type: lgn_source_control_proto::ChangeType) -> Result<Self> {
        Ok(if change_type.old_chunk_id.is_empty() {
            Self::Add {
                new_chunk_id: change_type
                    .new_chunk_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        } else if change_type.new_chunk_id.is_empty() {
            Self::Delete {
                old_chunk_id: change_type
                    .old_chunk_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        } else {
            Self::Edit {
                old_chunk_id: change_type
                    .old_chunk_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
                new_chunk_id: change_type
                    .new_chunk_id
                    .parse()
                    .map_other_err("reading chunk identifier")?,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use lgn_content_store2::Identifier;

    use super::*;

    fn id(data: &str) -> ChunkIdentifier {
        ChunkIdentifier::new(
            data.len().try_into().unwrap(),
            Identifier::new(data.as_bytes()),
        )
    }

    #[test]
    fn test_change_type_new() {
        assert_eq!(
            ChangeType::new(None, Some(id("new"))),
            Some(ChangeType::Add {
                new_chunk_id: id("new"),
            }),
        );
        assert_eq!(
            ChangeType::new(Some(id("old")), Some(id("new"))),
            Some(ChangeType::Edit {
                old_chunk_id: id("old"),
                new_chunk_id: id("new"),
            }),
        );
        assert_eq!(
            ChangeType::new(Some(id("old")), None),
            Some(ChangeType::Delete {
                old_chunk_id: id("old"),
            }),
        );
        assert_eq!(ChangeType::new(None, None), None,);
    }

    #[test]
    fn test_change_type_old_info() {
        assert_eq!(
            ChangeType::Add {
                new_chunk_id: id("new"),
            }
            .old_chunk_id(),
            None,
        );
        assert_eq!(
            ChangeType::Edit {
                old_chunk_id: id("old"),
                new_chunk_id: id("new"),
            }
            .old_chunk_id(),
            Some(&id("old")),
        );
        assert_eq!(
            ChangeType::Delete {
                old_chunk_id: id("old"),
            }
            .old_chunk_id(),
            Some(&id("old")),
        );
    }

    #[test]
    fn test_change_type_new_info() {
        assert_eq!(
            ChangeType::Add {
                new_chunk_id: id("new"),
            }
            .new_chunk_id(),
            Some(&id("new")),
        );
        assert_eq!(
            ChangeType::Edit {
                old_chunk_id: id("old"),
                new_chunk_id: id("new"),
            }
            .new_chunk_id(),
            Some(&id("new")),
        );
        assert_eq!(
            ChangeType::Delete {
                old_chunk_id: id("old"),
            }
            .new_chunk_id(),
            None
        );
    }

    #[test]
    fn test_change_type_from_proto() {
        let proto = lgn_source_control_proto::ChangeType {
            old_chunk_id: id("old").to_string(),
            new_chunk_id: id("new").to_string(),
        };

        let change_type = ChangeType::try_from(proto).unwrap();

        assert_eq!(
            change_type,
            ChangeType::Edit {
                old_chunk_id: id("old"),
                new_chunk_id: id("new"),
            }
        );
    }

    #[test]
    fn test_change_type_into_proto() {
        let change_type = ChangeType::Edit {
            old_chunk_id: id("old"),
            new_chunk_id: id("new"),
        };

        let proto: lgn_source_control_proto::ChangeType = change_type.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::ChangeType {
                old_chunk_id: id("old").to_string(),
                new_chunk_id: id("new").to_string(),
            }
        );
    }

    #[test]
    fn test_change_type_into_invert() {
        assert_eq!(
            ChangeType::Add {
                new_chunk_id: id("new"),
            }
            .into_invert(),
            ChangeType::Delete {
                old_chunk_id: id("new"),
            }
        );

        assert_eq!(
            ChangeType::Edit {
                old_chunk_id: id("old"),
                new_chunk_id: id("new"),
            }
            .into_invert(),
            ChangeType::Edit {
                old_chunk_id: id("new"),
                new_chunk_id: id("old"),
            }
        );

        assert_eq!(
            ChangeType::Delete {
                old_chunk_id: id("old"),
            }
            .into_invert(),
            ChangeType::Add {
                new_chunk_id: id("old"),
            }
        );
    }
}
