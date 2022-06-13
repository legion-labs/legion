use std::{collections::BTreeSet, num::ParseIntError, str::FromStr, time::SystemTime};

use chrono::{DateTime, NaiveDateTime, Utc};
use lgn_content_store::indexing::TreeIdentifier;
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
    pub main_index_tree_id: TreeIdentifier,
    pub path_index_tree_id: TreeIdentifier,
    pub parents: BTreeSet<CommitId>,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    #[span_fn]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CommitId,
        owner: String,
        message: String,
        changes: BTreeSet<Change>,
        main_index_tree_id: TreeIdentifier,
        path_index_tree_id: TreeIdentifier,
        parents: BTreeSet<CommitId>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        assert!(!parents.contains(&id), "commit cannot be its own parent");

        Self {
            id,
            owner,
            message,
            changes,
            main_index_tree_id,
            path_index_tree_id,
            parents,
            timestamp,
        }
    }

    #[span_fn]
    pub fn new_unique_now(
        owner: String,
        message: impl Into<String>,
        changes: BTreeSet<Change>,
        main_index_tree_id: TreeIdentifier,
        path_index_tree_id: TreeIdentifier,
        parents: BTreeSet<CommitId>,
    ) -> Self {
        let id = CommitId(0);
        let timestamp = Utc::now();

        Self::new(
            id,
            owner,
            message.into(),
            changes,
            main_index_tree_id,
            path_index_tree_id,
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
            main_index_tree_id: commit.main_index_tree_id.to_string(),
            path_index_tree_id: commit.path_index_tree_id.to_string(),
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
            main_index_tree_id: commit.main_index_tree_id.parse().unwrap(),
            path_index_tree_id: commit.path_index_tree_id.parse().unwrap(),
            parents: commit.parents.into_iter().map(CommitId).collect(),
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN_INDEX_TREE_ID: &str = "AG5vZGU2";
    const PATH_INDEX_TREE_ID: &str = "AG5vZGU3";

    #[test]
    fn test_commit_from_proto() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let now_sys = SystemTime::from(now);

        let proto = lgn_source_control_proto::Commit {
            id: 42,
            owner: "owner".to_owned(),
            message: "message".to_owned(),
            changes: vec![],
            main_index_tree_id: MAIN_INDEX_TREE_ID.to_owned(),
            path_index_tree_id: PATH_INDEX_TREE_ID.to_owned(),
            parents: vec![43],
            timestamp: Some(now_sys.into()),
        };

        let commit = Commit::try_from(proto).unwrap();

        assert_eq!(
            commit,
            Commit {
                id: CommitId(42),
                owner: "owner".to_owned(),
                message: "message".to_owned(),
                changes: BTreeSet::new(),
                main_index_tree_id: MAIN_INDEX_TREE_ID.parse().unwrap(),
                path_index_tree_id: PATH_INDEX_TREE_ID.parse().unwrap(),
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
            owner: "owner".to_owned(),
            message: "message".to_owned(),
            changes: BTreeSet::new(),
            main_index_tree_id: MAIN_INDEX_TREE_ID.parse().unwrap(),
            path_index_tree_id: PATH_INDEX_TREE_ID.parse().unwrap(),
            parents: vec![CommitId(43)].into_iter().collect(),
            timestamp: now,
        };

        let proto: lgn_source_control_proto::Commit = commit.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::Commit {
                id: 42,
                owner: "owner".to_owned(),
                message: "message".to_owned(),
                changes: vec![],
                main_index_tree_id: MAIN_INDEX_TREE_ID.to_owned(),
                path_index_tree_id: PATH_INDEX_TREE_ID.to_owned(),
                parents: vec![43],
                timestamp: Some(now_sys.into()),
            }
        );
    }
}
