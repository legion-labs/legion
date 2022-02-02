use std::{collections::BTreeSet, time::SystemTime};

use chrono::{DateTime, NaiveDateTime, Utc};
use lgn_tracing::span_fn;

use crate::{Change, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: String,
    pub owner: String,
    pub message: String,
    pub changes: BTreeSet<Change>,
    pub root_tree_id: String,
    pub parents: BTreeSet<String>,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    #[span_fn]
    pub fn new(
        id: String,
        owner: String,
        message: String,
        changes: BTreeSet<Change>,
        root_tree_id: String,
        parents: BTreeSet<String>,
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
        parents: BTreeSet<String>,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
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
            id: commit.id,
            owner: commit.owner,
            message: commit.message,
            changes: commit.changes.into_iter().map(Into::into).collect(),
            root_tree_id: commit.root_tree_id,
            parents: commit.parents.into_iter().collect(),
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
            id: commit.id,
            owner: commit.owner,
            message: commit.message,
            changes,
            root_tree_id: commit.root_tree_id,
            parents: commit.parents.into_iter().collect(),
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
            id: "id".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            changes: vec![],
            root_tree_id: "root_tree_id".to_string(),
            parents: vec!["parent".to_string()],
            timestamp: Some(now_sys.into()),
        };

        let commit = Commit::try_from(proto).unwrap();

        assert_eq!(
            commit,
            Commit {
                id: "id".to_string(),
                owner: "owner".to_string(),
                message: "message".to_string(),
                changes: BTreeSet::new(),
                root_tree_id: "root_tree_id".to_string(),
                parents: vec!["parent".to_string()].into_iter().collect(),
                timestamp: now,
            }
        );
    }

    #[test]
    fn test_commit_to_proto() {
        let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let now_sys = SystemTime::from(now);

        let commit = Commit {
            id: "id".to_string(),
            owner: "owner".to_string(),
            message: "message".to_string(),
            changes: BTreeSet::new(),
            root_tree_id: "root_tree_id".to_string(),
            parents: vec!["parent".to_string()].into_iter().collect(),
            timestamp: now,
        };

        let proto: lgn_source_control_proto::Commit = commit.into();

        assert_eq!(
            proto,
            lgn_source_control_proto::Commit {
                id: "id".to_string(),
                owner: "owner".to_string(),
                message: "message".to_string(),
                changes: vec![],
                root_tree_id: "root_tree_id".to_string(),
                parents: vec!["parent".to_string()],
                timestamp: Some(now_sys.into()),
            }
        );
    }
}
