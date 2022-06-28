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
pub struct WorkspaceId(String);

impl Display for WorkspaceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for WorkspaceId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidSpaceId(s.to_string()));
        }

        Ok(Self(s.to_string()))
    }
}

impl From<WorkspaceId> for crate::api::workspace::WorkspaceId {
    fn from(space_id: WorkspaceId) -> Self {
        Self(space_id.0)
    }
}

impl From<crate::api::workspace::WorkspaceId> for WorkspaceId {
    fn from(space_id: crate::api::workspace::WorkspaceId) -> Self {
        Self(space_id.0)
    }
}

/// A space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub last_updated_at: DateTime<Utc>,
}

impl From<Workspace> for crate::api::workspace::Workspace {
    fn from(workspace: Workspace) -> Self {
        Self {
            id: workspace.id.into(),
            name: workspace.name,
            description: workspace.description,
            created_at: workspace.created_at,
            last_updated_at: workspace.last_updated_at,
        }
    }
}

impl From<crate::api::workspace::Workspace> for Workspace {
    fn from(workspace: crate::api::workspace::Workspace) -> Self {
        Self {
            id: workspace.id.into(),
            name: workspace.name,
            description: workspace.description,
            created_at: workspace.created_at,
            last_updated_at: workspace.last_updated_at,
        }
    }
}
