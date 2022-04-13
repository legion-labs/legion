use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{Error, Result};

use super::IndexKey;

/// An index key.
///
/// The optimized no-alloc storage size is 16 bytes, which is the size of a
/// UUID. This is not a coincidence.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CompositeIndexKey(SmallVec<[IndexKey; 4]>);

impl Display for CompositeIndexKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl FromStr for CompositeIndexKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').map(str::parse).collect::<Result<_>>()?;

        Ok(Self(parts))
    }
}

impl Deref for CompositeIndexKey {
    type Target = SmallVec<[IndexKey; 4]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[IndexKey]> for CompositeIndexKey {
    fn as_ref(&self) -> &[IndexKey] {
        &self.0
    }
}

impl<T: Into<SmallVec<[IndexKey; 4]>>> From<T> for CompositeIndexKey {
    fn from(keys: T) -> Self {
        Self(keys.into())
    }
}
