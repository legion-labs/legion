use std::time::SystemTime;

use chrono::{DateTime, NaiveDateTime, Utc};
use lgn_tracing::span_fn;

use super::HashedChange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: String,
    pub owner: String,
    pub message: String,
    pub changes: Vec<HashedChange>,
    pub root_hash: String,
    pub parents: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    #[span_fn]
    pub fn new(
        id: String,
        owner: String,
        message: String,
        changes: Vec<HashedChange>,
        root_hash: String,
        parents: Vec<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        assert!(!parents.contains(&id));
        Self {
            id,
            owner,
            message,
            changes,
            root_hash,
            parents,
            timestamp,
        }
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
            root_hash: commit.root_hash,
            parents: commit.parents,
            timestamp: Some(timestamp.into()),
        }
    }
}

impl TryFrom<lgn_source_control_proto::Commit> for Commit {
    type Error = anyhow::Error;

    fn try_from(commit: lgn_source_control_proto::Commit) -> anyhow::Result<Self> {
        let timestamp = commit.timestamp.unwrap_or_default();
        let timestamp = DateTime::from_utc(
            NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32),
            Utc,
        );

        let changes = commit
            .changes
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            id: commit.id,
            owner: commit.owner,
            message: commit.message,
            changes,
            root_hash: commit.root_hash,
            parents: commit.parents,
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
            root_hash: "root_hash".to_string(),
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
                changes: vec![],
                root_hash: "root_hash".to_string(),
                parents: vec!["parent".to_string()],
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
            changes: vec![],
            root_hash: "root_hash".to_string(),
            parents: vec!["parent".to_string()],
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
                root_hash: "root_hash".to_string(),
                parents: vec!["parent".to_string()],
                timestamp: Some(now_sys.into()),
            }
        );
    }
}
