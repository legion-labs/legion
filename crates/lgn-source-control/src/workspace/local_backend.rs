use async_trait::async_trait;
use lgn_tracing::prelude::*;
use sqlx::{migrate::MigrateDatabase, Connection, Executor, Row};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

use crate::{
    Branch, CanonicalPath, Change, ChangeType, CommitId, MapOtherError, PendingBranchMerge,
    ResolvePending, Result,
};

use super::WorkspaceBackend;

pub struct LocalWorkspaceBackend {
    sql_connection: Mutex<sqlx::AnyConnection>,
}

impl LocalWorkspaceBackend {
    const TABLE_CONFIG: &'static str = "config";
    const TABLE_CHANGES: &'static str = "changes";
    const TABLE_RESOLVES_PENDING: &'static str = "resolves_pending";
    const TABLE_BRANCH_MERGES_PENDING: &'static str = "branch_merges_pending";

    const CONFIG_CURRENT_BRANCH: &'static str = "current-branch";

    #[span_fn]
    pub async fn create(root: &PathBuf) -> Result<Self> {
        let db_uri = Self::db_uri(root);

        sqlx::Any::create_database(&db_uri)
            .await
            .map_other_err("failed to create workspace database")?;

        let mut workspace = Self::connect(root).await?;

        workspace.create_config_table().await?;
        workspace.create_changes_table().await?;
        workspace.create_resolves_pending_table().await?;
        workspace.create_branch_merges_pending_table().await?;

        Ok(workspace)
    }

    #[span_fn]
    pub async fn connect(root: &PathBuf) -> Result<Self> {
        let db_uri = Self::db_uri(root);

        let sql_connection = Mutex::new(
            sqlx::AnyConnection::connect(&db_uri)
                .await
                .map_other_err("failed to connect to the database")?,
        );

        Ok(Self { sql_connection })
    }

    fn db_uri(lsc_root: impl AsRef<Path>) -> String {
        format!(
            "sqlite://{}",
            lsc_root.as_ref().join("workspace.db3").display()
        )
    }

    #[span_fn]
    async fn create_config_table(&mut self) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}`(key VARCHAR(255) NOT NULL PRIMARY KEY, value VARCHAR(8192) NOT NULL, unique(key));",
            Self::TABLE_CONFIG
        );

        self.sql_connection
            .lock()
            .await
            .execute(sql)
            .await
            .map_other_err("failed to create current branch table")
            .map(|_| ())
    }

    #[span_fn]
    async fn create_changes_table(&mut self) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}`(canonical_path TEXT NOT NULL PRIMARY KEY, old_cs_id VARCHAR(255), new_cs_id VARCHAR(255), unique(canonical_path));",
            Self::TABLE_CHANGES
        );

        self.sql_connection
            .lock()
            .await
            .execute(sql)
            .await
            .map_other_err("failed to create changes table")
            .map(|_| ())
    }

    #[span_fn]
    async fn create_resolves_pending_table(&mut self) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}`(canonical_path VARCHAR(512) NOT NULL PRIMARY KEY, base_commit_id VARCHAR(255), theirs_commit_id VARCHAR(255), unique(canonical_path));",
            Self::TABLE_RESOLVES_PENDING
        );

        self.sql_connection
            .lock()
            .await
            .execute(sql)
            .await
            .map_other_err("failed to create resolves pending table")
            .map(|_| ())
    }

    #[span_fn]
    async fn create_branch_merges_pending_table(&mut self) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}`(name VARCHAR(255) NOT NULL PRIMARY KEY, head INTEGER, unique(name));",
            Self::TABLE_BRANCH_MERGES_PENDING,
        );

        self.sql_connection
            .lock()
            .await
            .execute(sql)
            .await
            .map_other_err("failed to create branch merges pending table")
            .map(|_| ())
    }
}

#[async_trait]
impl WorkspaceBackend for LocalWorkspaceBackend {
    async fn get_current_branch(&self) -> Result<Branch> {
        async_span_scope!("LocalWorkspaceBackend::get_current_branch");
        let sql: &str = &format!("SELECT value FROM `{}` WHERE key = ?;", Self::TABLE_CONFIG);
        let sql = sqlx::query(sql).bind(Self::CONFIG_CURRENT_BRANCH);

        let mut conn = self.sql_connection.lock().await;

        let row = conn
            .fetch_one(sql)
            .await
            .map_other_err("failed to get current branch")?;

        serde_json::from_str(row.get("value"))
            .map_other_err("failed to deserialize current branch information")
    }

    async fn set_current_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::set_current_branch");
        let value = serde_json::to_string(&branch)
            .map_other_err("failed to serialize current branch information")?;

        let sql: &str = &format!(
            "REPLACE INTO `{}` (key, value) VALUES(?, ?);",
            Self::TABLE_CONFIG
        );
        let sql = sqlx::query(sql)
            .bind(Self::CONFIG_CURRENT_BRANCH)
            .bind(value);

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to set current branch")
            .map(|_| ())
    }

    async fn get_staged_changes(&self) -> Result<BTreeMap<CanonicalPath, Change>> {
        async_span_scope!("LocalWorkspaceBackend::get_staged_changes");
        let sql: &str = &format!(
            "SELECT canonical_path, old_cs_id, new_cs_id FROM {}",
            Self::TABLE_CHANGES
        );

        let mut conn = self.sql_connection.lock().await;

        let rows = conn
            .fetch_all(sql)
            .await
            .map_other_err("failed to set current branch")?;

        drop(conn);

        let mut res = BTreeMap::new();

        for row in rows {
            let old_cs_id: String = row.get("old_cs_id");

            let old_cs_id = if !old_cs_id.is_empty() {
                Some(
                    old_cs_id
                        .parse()
                        .map_other_err("failed to parse old chunk id")?,
                )
            } else {
                None
            };

            let new_cs_id: String = row.get("new_cs_id");

            let new_cs_id = if !new_cs_id.is_empty() {
                Some(
                    new_cs_id
                        .parse()
                        .map_other_err("failed to parse new chunk id")?,
                )
            } else {
                None
            };

            let canonical_path = CanonicalPath::new(row.get("canonical_path"))?;

            if let Some(change_type) = ChangeType::new(old_cs_id, new_cs_id) {
                res.insert(
                    canonical_path.clone(),
                    Change::new(canonical_path, change_type),
                );
            }
        }

        Ok(res)
    }

    async fn save_staged_changes(&self, changes: &[Change]) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::save_staged_changes");
        if changes.is_empty() {
            return Ok(());
        }

        let sql: &str = &format!(
            "REPLACE INTO `{}` (canonical_path, old_cs_id, new_cs_id) VALUES(?, ?, ?);",
            Self::TABLE_CHANGES
        );

        let mut conn = self.sql_connection.lock().await;
        let mut transaction = conn
            .begin()
            .await
            .map_other_err("failed to start transaction")?;

        for change in changes {
            let sql = sqlx::query(sql)
                .bind(change.canonical_path().to_string())
                .bind(
                    change
                        .change_type()
                        .old_id()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default(),
                )
                .bind(
                    change
                        .change_type()
                        .new_id()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default(),
                );

            transaction.execute(sql).await.map_other_err(format!(
                "failed to save staged change `{}`",
                change.canonical_path()
            ))?;
        }

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction")?;

        Ok(())
    }

    async fn clear_staged_changes(&self, changes: &[Change]) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::clear_staged_changes");
        let mut conn = self.sql_connection.lock().await;
        let mut transaction = conn
            .begin()
            .await
            .map_other_err("failed to begin transaction")?;

        for change in changes {
            let sql: &str = &format!(
                "DELETE from {} where canonical_path=?;",
                Self::TABLE_CHANGES
            );
            let sql = sqlx::query(sql).bind(change.canonical_path().to_string());

            transaction
                .execute(sql)
                .await
                .map_other_err("failed to clear local changes")?;
        }

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction")
            .map(|_| ())
    }

    async fn read_pending_branch_merges(&self) -> Result<Vec<PendingBranchMerge>> {
        async_span_scope!("LocalWorkspaceBackend::read_pending_branch_merges");
        let sql: &str = &format!(
            "SELECT name, head FROM {};",
            Self::TABLE_BRANCH_MERGES_PENDING
        );

        let mut conn = self.sql_connection.lock().await;

        conn.fetch_all(sql)
            .await
            .map_other_err("failed to read pending branch merges")?
            .into_iter()
            .map(|row| {
                Ok(PendingBranchMerge {
                    name: row.get("name"),
                    head: CommitId(
                        row.get::<i64, _>("head")
                            .try_into()
                            .map_other_err("failed to read commit head")?,
                    ),
                })
            })
            .collect()
    }

    async fn clear_pending_branch_merges(&self) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::clear_pending_branch_merges");
        let sql: &str = &format!("DELETE from {};", Self::TABLE_BRANCH_MERGES_PENDING);

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to clear pending branch merges")
            .map(|_| ())
    }

    async fn save_pending_branch_merge(&self, merge_spec: &PendingBranchMerge) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::save_pending_branch_merge");
        let sql: &str = &format!(
            "INSERT OR REPLACE into {} VALUES(?,?);",
            Self::TABLE_BRANCH_MERGES_PENDING
        );

        let head: i64 = merge_spec
            .head
            .0
            .try_into()
            .map_other_err("failed to read commit head")?;

        let sql = sqlx::query(sql).bind(merge_spec.name.clone()).bind(head);

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to save pending branch merge")
            .map(|_| ())
    }

    async fn save_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::save_resolve_pending");
        let sql: &str = &format!(
            "INSERT OR REPLACE into {} VALUES(?,?,?);",
            Self::TABLE_RESOLVES_PENDING
        );
        let sql = sqlx::query(sql)
            .bind(resolve_pending.relative_path.clone())
            .bind(resolve_pending.base_commit_id.clone())
            .bind(resolve_pending.theirs_commit_id.clone());

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to save resolve pending")
            .map(|_| ())
    }

    async fn clear_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()> {
        async_span_scope!("LocalWorkspaceBackend::clear_resolve_pending");
        let sql: &str = &format!(
            "DELETE from {}
             WHERE canonical_path=?;",
            Self::TABLE_RESOLVES_PENDING
        );
        let sql = sqlx::query(sql).bind(resolve_pending.relative_path.clone());

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to save resolve pending")
            .map(|_| ())
    }

    async fn find_resolve_pending(&self, canonical_path: &str) -> Result<Option<ResolvePending>> {
        async_span_scope!("LocalWorkspaceBackend::find_resolve_pending");
        let sql: &str = &format!(
            "SELECT base_commit_id, theirs_commit_id 
             FROM {}
             WHERE canonical_path = ?;",
            Self::TABLE_RESOLVES_PENDING
        );
        let sql = sqlx::query(sql).bind(canonical_path);

        let mut conn = self.sql_connection.lock().await;

        Ok(conn
            .fetch_optional(sql)
            .await
            .map_other_err("failed to find resolve pending")?
            .map(|row| {
                ResolvePending::new(
                    String::from(canonical_path),
                    row.get("base_commit_id"),
                    row.get("theirs_commit_id"),
                )
            }))
    }

    async fn read_resolves_pending(&self) -> Result<Vec<ResolvePending>> {
        async_span_scope!("LocalWorkspaceBackend::read_resolves_pending");
        let sql: &str = &format!(
            "SELECT canonical_path, base_commit_id, theirs_commit_id 
             FROM {};",
            Self::TABLE_RESOLVES_PENDING
        );

        let mut conn = self.sql_connection.lock().await;

        Ok(conn
            .fetch_all(sql)
            .await
            .map_other_err("failed to fetch resolves pending")?
            .into_iter()
            .map(|row| {
                ResolvePending::new(
                    row.get("canonical_path"),
                    row.get("base_commit_id"),
                    row.get("theirs_commit_id"),
                )
            })
            .collect())
    }
}
