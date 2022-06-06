use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
        Ok(Self(s.to_string()))
    }
}

impl From<SpaceId> for crate::api::common::SpaceId {
    fn from(space_id: SpaceId) -> Self {
        Self(space_id.0)
    }
}

impl From<crate::api::common::SpaceId> for SpaceId {
    fn from(space_id: crate::api::common::SpaceId) -> Self {
        Self(space_id.0)
    }
}

/// A space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    pub id: SpaceId,
    pub description: String,
    pub cordoned: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Space> for crate::api::common::Space {
    fn from(space: Space) -> Self {
        Self {
            id: space.id.into(),
            description: space.description,
            cordoned: space.cordoned,
            created_at: space.created_at,
        }
    }
}

impl From<crate::api::common::Space> for Space {
    fn from(space: crate::api::common::Space) -> Self {
        Self {
            id: space.id.into(),
            description: space.description,
            cordoned: space.cordoned,
            created_at: space.created_at,
        }
    }
}
