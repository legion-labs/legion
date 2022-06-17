use std::{collections::BTreeSet, num::ParseIntError, str::FromStr};

use chrono::{DateTime, Utc};
use lgn_content_store::indexing::TreeIdentifier;
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

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

impl From<CommitId> for crate::api::source_control::CommitId {
    fn from(id: CommitId) -> Self {
        Self(id.0)
    }
}

impl From<crate::api::source_control::CommitId> for CommitId {
    fn from(id: crate::api::source_control::CommitId) -> Self {
        Self(id.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: CommitId,
    pub owner: String,
    pub message: String,
    pub main_index_tree_id: TreeIdentifier,
    pub path_index_tree_id: TreeIdentifier,
    pub parents: BTreeSet<CommitId>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCommit {
    pub owner: String,
    pub message: String,
    pub main_index_tree_id: TreeIdentifier,
    pub path_index_tree_id: TreeIdentifier,
    pub parents: BTreeSet<CommitId>,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    #[span_fn]
    pub fn new(
        id: CommitId,
        owner: String,
        message: String,
        main_index_tree_id: TreeIdentifier,
        path_index_tree_id: TreeIdentifier,
        parents: BTreeSet<CommitId>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            owner,
            message,
            main_index_tree_id,
            path_index_tree_id,
            parents,
            timestamp,
        }
    }
}

impl NewCommit {
    #[span_fn]
    pub fn new(
        owner: String,
        message: String,
        main_index_tree_id: TreeIdentifier,
        path_index_tree_id: TreeIdentifier,
        parents: BTreeSet<CommitId>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            owner,
            message,
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
        main_index_tree_id: TreeIdentifier,
        path_index_tree_id: TreeIdentifier,
        parents: BTreeSet<CommitId>,
    ) -> Self {
        let timestamp = Utc::now();

        Self::new(
            owner,
            message.into(),
            main_index_tree_id,
            path_index_tree_id,
            parents,
            timestamp,
        )
    }

    pub fn into_commit(self, id: CommitId) -> Commit {
        Commit::new(
            id,
            self.owner,
            self.message,
            self.main_index_tree_id,
            self.path_index_tree_id,
            self.parents,
            self.timestamp,
        )
    }
}

impl From<Commit> for crate::api::source_control::Commit {
    fn from(commit: Commit) -> Self {
        Self {
            id: commit.id.into(),
            owner: commit.owner,
            message: commit.message,
            main_index_tree_id: commit.main_index_tree_id.to_string(),
            path_index_tree_id: commit.path_index_tree_id.to_string(),
            parents: commit.parents.into_iter().map(Into::into).collect(),
            timestamp: commit.timestamp,
        }
    }
}

impl TryFrom<crate::api::source_control::Commit> for Commit {
    type Error = Error;

    fn try_from(commit: crate::api::source_control::Commit) -> Result<Self> {
        Ok(Self {
            id: commit.id.into(),
            owner: commit.owner,
            message: commit.message,
            main_index_tree_id: commit.main_index_tree_id.parse().unwrap(),
            path_index_tree_id: commit.path_index_tree_id.parse().unwrap(),
            parents: commit.parents.into_iter().map(Into::into).collect(),
            timestamp: commit.timestamp,
        })
    }
}

impl TryFrom<crate::api::source_control::NewCommit> for NewCommit {
    type Error = Error;

    fn try_from(commit: crate::api::source_control::NewCommit) -> Result<Self> {
        Ok(Self {
            owner: commit.owner,
            message: commit.message,
            main_index_tree_id: commit.main_index_tree_id.parse().unwrap(),
            path_index_tree_id: commit.path_index_tree_id.parse().unwrap(),
            parents: commit.parents.into_iter().map(Into::into).collect(),
            timestamp: commit.timestamp,
        })
    }
}

impl From<NewCommit> for crate::api::source_control::NewCommit {
    fn from(commit: NewCommit) -> Self {
        Self {
            owner: commit.owner,
            message: commit.message,
            main_index_tree_id: commit.main_index_tree_id.to_string(),
            path_index_tree_id: commit.path_index_tree_id.to_string(),
            parents: commit.parents.into_iter().map(Into::into).collect(),
            timestamp: commit.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN_INDEX_TREE_ID: &str = "AG5vZGU2";
    const PATH_INDEX_TREE_ID: &str = "AG5vZGU3";

    #[test]
    fn test_commit_from_api() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

        let api = crate::api::source_control::Commit {
            id: crate::api::source_control::CommitId(42),
            owner: "owner".to_owned(),
            message: "message".to_owned(),
            main_index_tree_id: MAIN_INDEX_TREE_ID.to_owned(),
            path_index_tree_id: PATH_INDEX_TREE_ID.to_owned(),
            parents: vec![crate::api::source_control::CommitId(43)],
            timestamp: now,
        };

        let commit = Commit::try_from(api).unwrap();

        assert_eq!(
            commit,
            Commit {
                id: CommitId(42),
                owner: "owner".to_owned(),
                message: "message".to_owned(),
                main_index_tree_id: MAIN_INDEX_TREE_ID.parse().unwrap(),
                path_index_tree_id: PATH_INDEX_TREE_ID.parse().unwrap(),
                parents: vec![CommitId(43)].into_iter().collect(),
                timestamp: now,
            }
        );
    }

    #[test]
    fn test_commit_to_api() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

        let commit = Commit {
            id: CommitId(42),
            owner: "owner".to_owned(),
            message: "message".to_owned(),
            main_index_tree_id: MAIN_INDEX_TREE_ID.parse().unwrap(),
            path_index_tree_id: PATH_INDEX_TREE_ID.parse().unwrap(),
            parents: vec![CommitId(43)].into_iter().collect(),
            timestamp: now,
        };

        let api: crate::api::source_control::Commit = commit.into();

        assert_eq!(
            api,
            crate::api::source_control::Commit {
                id: crate::api::source_control::CommitId(42),
                owner: "owner".to_owned(),
                message: "message".to_owned(),
                main_index_tree_id: MAIN_INDEX_TREE_ID.to_owned(),
                path_index_tree_id: PATH_INDEX_TREE_ID.to_owned(),
                parents: vec![crate::api::source_control::CommitId(43)],
                timestamp: now,
            }
        );
    }
}
