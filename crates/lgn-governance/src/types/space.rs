use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "tabled")]
use tabled::Tabled;

use super::{Error, Result};

/// A space identifier.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct SpaceId(String);

impl Display for SpaceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SpaceId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() < 3 {
            return Err(Error::InvalidSpaceId(s.to_string()));
        }

        if s.contains(|c: char| !matches!(c, '0'..='9' | 'a'..='z' | '_' | '-')) {
            return Err(Error::InvalidSpaceId(s.to_string()));
        }

        Ok(Self(s.to_string()))
    }
}

impl From<SpaceId> for crate::api::space::SpaceId {
    fn from(space_id: SpaceId) -> Self {
        Self(space_id.0)
    }
}

impl From<crate::api::space::SpaceId> for SpaceId {
    fn from(space_id: crate::api::space::SpaceId) -> Self {
        Self(space_id.0)
    }
}

/// A space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Space {
    pub id: SpaceId,
    pub description: String,
    pub cordoned: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Space> for crate::api::space::Space {
    fn from(space: Space) -> Self {
        Self {
            id: space.id.into(),
            description: space.description,
            cordoned: space.cordoned,
            created_at: space.created_at,
        }
    }
}

impl From<crate::api::space::Space> for Space {
    fn from(space: crate::api::space::Space) -> Self {
        Self {
            id: space.id.into(),
            description: space.description,
            cordoned: space.cordoned,
            created_at: space.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceUpdate {
    pub description: Option<String>,
}

impl From<SpaceUpdate> for crate::api::space::SpaceUpdate {
    fn from(space: SpaceUpdate) -> Self {
        Self {
            description: space.description,
        }
    }
}

impl From<crate::api::space::SpaceUpdate> for SpaceUpdate {
    fn from(space: crate::api::space::SpaceUpdate) -> Self {
        Self {
            description: space.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_id() {
        // Build a few valid space ids.
        let space_id = SpaceId::from_str("abc").unwrap();
        assert_eq!(space_id.0, "abc");

        let space_id = SpaceId::from_str("some_space_id").unwrap();
        assert_eq!(space_id.0, "some_space_id");

        let space_id = SpaceId::from_str("i_contain_numb-3rs").unwrap();
        assert_eq!(space_id.0, "i_contain_numb-3rs");

        // Build a few invalid space ids.
        assert!(SpaceId::from_str("").is_err());
        assert!(SpaceId::from_str("a").is_err());
        assert!(SpaceId::from_str("ABC").is_err());
        assert!(SpaceId::from_str("abc~d").is_err());
        assert!(SpaceId::from_str("ab c").is_err());
    }
}
