use std::fmt::Display;

use lgn_content_store::indexing::ResourceIdentifier;

use crate::{Error, MapOtherError, Result};

/// A change type for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]

pub enum ChangeType {
    Add {
        new_id: ResourceIdentifier,
    },
    Edit {
        old_id: ResourceIdentifier,
        new_id: ResourceIdentifier,
    },
    Delete {
        old_id: ResourceIdentifier,
    },
}

impl ChangeType {
    pub fn new(
        old_id: Option<ResourceIdentifier>,
        new_id: Option<ResourceIdentifier>,
    ) -> Option<Self> {
        match (old_id, new_id) {
            (Some(old_id), Some(new_id)) => Some(Self::Edit { old_id, new_id }),
            (Some(old_id), None) => Some(Self::Delete { old_id }),
            (None, Some(new_id)) => Some(Self::Add { new_id }),
            (None, None) => None,
        }
    }

    pub fn old_id(&self) -> Option<&ResourceIdentifier> {
        match self {
            ChangeType::Add { .. } => None,
            ChangeType::Edit { old_id, .. } | ChangeType::Delete { old_id } => Some(old_id),
        }
    }

    pub fn new_id(&self) -> Option<&ResourceIdentifier> {
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

impl From<ChangeType> for crate::api::source_control::ChangeType {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Add { new_id } => Self {
                old_id: None,
                new_id: Some(new_id.to_string()),
            },
            ChangeType::Edit { old_id, new_id } => Self {
                old_id: Some(old_id.to_string()),
                new_id: Some(new_id.to_string()),
            },
            ChangeType::Delete { old_id } => Self {
                old_id: Some(old_id.to_string()),
                new_id: None,
            },
        }
    }
}

impl TryFrom<crate::api::source_control::ChangeType> for ChangeType {
    type Error = Error;

    fn try_from(change_type: crate::api::source_control::ChangeType) -> Result<Self> {
        Ok(match change_type {
            crate::api::source_control::ChangeType {
                old_id: None,
                new_id: Some(new_id),
            } => ChangeType::Add {
                new_id: new_id.parse().map_other_err("reading chunk identifier")?,
            },
            crate::api::source_control::ChangeType {
                old_id: Some(old_id),
                new_id: Some(new_id),
            } => ChangeType::Edit {
                old_id: old_id.parse().map_other_err("reading chunk identifier")?,
                new_id: new_id.parse().map_other_err("reading chunk identifier")?,
            },
            crate::api::source_control::ChangeType {
                old_id: Some(old_id),
                new_id: None,
            } => ChangeType::Delete {
                old_id: old_id.parse().map_other_err("reading chunk identifier")?,
            },
            _ => return Err(Error::InvalidChangeType),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use lgn_content_store::Provider;

    use super::*;

    fn id(data: &str) -> ResourceIdentifier {
        let id = Provider::new_in_memory().compute_id(data.as_bytes());
        let id_as_str = format!("{}", id);
        ResourceIdentifier::from_str(id_as_str.as_str()).expect("failed to parse")
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
    fn test_change_type_from_api() {
        let api = crate::api::source_control::ChangeType {
            old_id: Some(id("old").to_string()),
            new_id: Some(id("new").to_string()),
        };

        let change_type = ChangeType::try_from(api).unwrap();

        assert_eq!(
            change_type,
            ChangeType::Edit {
                old_id: id("old"),
                new_id: id("new"),
            }
        );

        let api = crate::api::source_control::ChangeType {
            old_id: Some(id("old").to_string()),
            new_id: None,
        };

        let change_type = ChangeType::try_from(api).unwrap();

        assert_eq!(change_type, ChangeType::Delete { old_id: id("old") });

        let api = crate::api::source_control::ChangeType {
            old_id: None,
            new_id: Some(id("new").to_string()),
        };

        let change_type = ChangeType::try_from(api).unwrap();

        assert_eq!(change_type, ChangeType::Add { new_id: id("new") });
    }

    #[test]
    fn test_change_type_into_api() {
        let change_type = ChangeType::Edit {
            old_id: id("old"),
            new_id: id("new"),
        };

        let api: crate::api::source_control::ChangeType = change_type.into();

        assert_eq!(
            api,
            crate::api::source_control::ChangeType {
                old_id: Some(id("old").to_string()),
                new_id: Some(id("new").to_string()),
            }
        );

        let change_type = ChangeType::Add { new_id: id("new") };

        let api: crate::api::source_control::ChangeType = change_type.into();

        assert_eq!(
            api,
            crate::api::source_control::ChangeType {
                old_id: None,
                new_id: Some(id("new").to_string()),
            }
        );

        let change_type = ChangeType::Delete { old_id: id("old") };

        let api: crate::api::source_control::ChangeType = change_type.into();

        assert_eq!(
            api,
            crate::api::source_control::ChangeType {
                old_id: Some(id("old").to_string()),
                new_id: None,
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
