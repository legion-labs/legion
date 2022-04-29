use std::{fmt::Display, ops::Deref};

use smallvec::SmallVec;

/// Represents an alias.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alias(pub(crate) SmallVec<[u8; KEY_SIZE]>);

pub(crate) const KEY_SIZE: usize = 64;

impl Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl Deref for Alias {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Alias {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for Alias {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<Vec<u8>> for Alias {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
    }
}
