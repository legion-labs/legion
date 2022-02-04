use std::fmt::Display;

use super::{CanonicalPath, ChangeType};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Change {
    canonical_path: CanonicalPath,
    change_type: ChangeType,
}

impl From<Change> for lgn_source_control_proto::Change {
    fn from(change: Change) -> Self {
        Self {
            canonical_path: change.canonical_path.to_string(),
            change_type: Some(change.change_type.into()),
        }
    }
}

impl TryFrom<lgn_source_control_proto::Change> for Change {
    type Error = Error;

    fn try_from(change: lgn_source_control_proto::Change) -> Result<Self> {
        Ok(Self {
            canonical_path: CanonicalPath::new(&change.canonical_path)?,
            change_type: change
                .change_type
                .ok_or(Error::InvalidChangeType)?
                .try_into()?,
        })
    }
}

impl Change {
    pub fn new(canonical_path: CanonicalPath, change_type: ChangeType) -> Self {
        Self {
            canonical_path,
            change_type,
        }
    }

    pub fn canonical_path(&self) -> &CanonicalPath {
        &self.canonical_path
    }

    pub fn change_type(&self) -> &ChangeType {
        &self.change_type
    }

    pub fn into_invert(self) -> Self {
        Self {
            canonical_path: self.canonical_path,
            change_type: self.change_type.into_invert(),
        }
    }

    pub fn with_parent_name(self, parent_name: &str) -> Self {
        Self {
            canonical_path: self.canonical_path.prepend(parent_name),
            change_type: self.change_type,
        }
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.change_type, self.canonical_path)
    }
}

impl From<Change> for CanonicalPath {
    fn from(staged_change: Change) -> Self {
        staged_change.canonical_path
    }
}

#[cfg(test)]
mod tests {
    use crate::FileInfo;

    use super::*;

    fn sc(p: &str, ct: ChangeType) -> Change {
        Change::new(CanonicalPath::new(p).unwrap(), ct)
    }

    fn fi(hash: &str, size: u64) -> FileInfo {
        FileInfo {
            hash: hash.to_string(),
            size,
        }
    }

    fn add() -> ChangeType {
        ChangeType::Add {
            new_info: fi("new", 123),
        }
    }

    fn edit() -> ChangeType {
        ChangeType::Edit {
            old_info: fi("old", 123),
            new_info: fi("new", 123),
        }
    }

    #[test]
    fn test_staged_change_comparison() {
        assert_eq!(sc("/a", add()), sc("/a", add()));
        assert_ne!(sc("/a", add()), sc("/a", edit()));
    }

    #[test]
    fn test_staged_change_ordering() {
        assert!(sc("/a", add()) <= sc("/a", add()));
        assert!(sc("/a", add()) >= sc("/a", add()));
        assert!(sc("/a", add()) <= sc("/a", edit()));
        assert!(sc("/a", add()) < sc("/a", edit()));
        assert!(sc("/a", edit()) < sc("/b", add()));
    }
}
