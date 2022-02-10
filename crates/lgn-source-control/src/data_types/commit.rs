use std::{collections::BTreeSet, num::ParseIntError, str::FromStr, time::SystemTime};

use chrono::{DateTime, NaiveDateTime, Utc};
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};

use crate::{Change, Error, Result};

/// The ID for a commit.
///
/// A unsigned int 64 should be enough: if we make a billion commits per second,
/// it will take 583 years to overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitId(pub u64);

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CommitId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, ParseIntError> {
        let id = s.parse::<u64>()?;

        Ok(Self(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: CommitId,
    pub owner: String,
    pub message: String,
    pub changes: BTreeSet<Change>,
    pub root_tree_id: String,
    pub parents: BTreeSet<CommitId>,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    #[span_fn]
    pub fn new(
        id: CommitId,
        owner: String,
        message: String,
        changes: BTreeSet<Change>,
        root_tree_id: String,
        parents: BTreeSet<CommitId>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        assert!(!parents.contains(&id), "commit cannot be its own parent");

        Self {
            id,
            owner,
            message,
            changes,
            root_tree_id,
            parents,
            timestamp,
        }
    }

    #[span_fn]
    pub fn new_unique_now(
        owner: String,
        message: impl Into<String>,
        changes: BTreeSet<Change>,
        root_tree_id: String,
        parents: BTreeSet<CommitId>,
    ) -> Self {
        let id = CommitId(0);
        let timestamp = Utc::now();

        Self::new(
            id,
            owner,
            message.into(),
            changes,
            root_tree_id,
            parents,
            timestamp,
        )
    }
}

impl From<Commit> for lgn_source_control_proto::Commit {
    fn from(commit: Commit) -> Self {
        let timestamp: SystemTime = commit.timestamp.into();

        Self {
            id: commit.id.0,
            owner: commit.owner,
            message: commit.message,
            changes: commit.changes.into_iter().map(Into::into).collect(),
            root_tree_id: commit.root_tree_id,
            parents: commit.parents.into_iter().map(|id| id.0).collect(),
            timestamp: Some(timestamp.into()),
        }
    }
}

impl TryFrom<lgn_source_control_proto::Commit> for Commit {
    type Error = Error;

    fn try_from(commit: lgn_source_control_proto::Commit) -> Result<Self> {
        let timestamp = commit.timestamp.unwrap_or_default();
        let timestamp = DateTime::from_utc(
            NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32),
            Utc,
        );

        let changes = commit
            .changes
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<BTreeSet<Change>>>()?;

        Ok(Self {
            id: CommitId(commit.id),
            owner: commit.owner,
            message: commit.message,
            changes,
            root_tree_id: commit.root_tree_id,
            parents: commit.parents.into_iter().map(CommitId).collect(),
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_from_proto() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let now_sys = SystemTime::from(now);

        let proto = lgn_source_control_proto::Commit {
            id: 42,
            owner: "owner".to_string(),
            message: "message".to_string(),
            changes: vec![],
            root_tree_id: "root_tree_id".to_string(),
            parents: vec![43],
            timestamp: Some(now_sys.into()),
        };

        let commit = Commit::try_from(proto).unwrap();

        assert_eq!(
            commit,
            Commit {
                id: CommitId(42),
                owner: "owner".to_string(),
                message: "message".to_string(),
                changes: BTreeSet::new(),
                root_tree_id: "root_tree_id".to_string(),
                parents: vec![CommitId(43)].into_iter().collect(),
                timestamp: now,
            }
        );
    }

    #[test]
    fn test_commit_to_proto() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let now_sys = SystemTime::from(now);

        let commit = Commit {
            id: CommitId(42),
            owner: "owner".to_string(),
            message: "message".to_string(),
            changes: BTreeSet::new(),
            root_tree_id: "root_tree_id".to_string(),
            parents: vec![CommitId(43)].into_iter().collect(),
            timestamp: now,
        };

        let proto: lgn_source_control_proto::Commit = commit.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::Commit {
                id: 42,
                owner: "owner".to_string(),
                message: "message".to_string(),
                changes: vec![],
                root_tree_id: "root_tree_id".to_string(),
                parents: vec![43],
                timestamp: Some(now_sys.into()),
            }
        );
    }
}
