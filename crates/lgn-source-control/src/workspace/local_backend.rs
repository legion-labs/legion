use async_trait::async_trait;
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, Row};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

use crate::{
    sql::create_database, ChangeType, LocalChange, MapOtherError, PendingBranchMerge,
    ResolvePending, Result,
};

use super::WorkspaceBackend;

pub struct LocalWorkspaceBackend {
    sql_connection: Mutex<sqlx::AnyConnection>,
}

#[derive(Serialize, Deserialize)]
struct CurrentBranch {
    branch_name: String,
    commit_id: String,
}

impl LocalWorkspaceBackend {
    const TABLE_CONFIG: &'static str = "config";
    const TABLE_CHANGES: &'static str = "changes";
    const TABLE_RESOLVES_PENDING: &'static str = "resolves_pending";
    const TABLE_BRANCH_MERGES_PENDING: &'static str = "branch_merges_pending";

    const CONFIG_CURRENT_BRANCH: &'static str = "current-branch";

    #[span_fn]
    pub async fn create(lsc_root: PathBuf) -> Result<Self> {
        let db_uri = Self::db_uri(&lsc_root);

        create_database(&db_uri)
            .await
            .map_other_err("failed to create workspace database")?;

        let mut workspace = Self::connect(lsc_root).await?;

        workspace.create_config_table().await?;
        workspace.create_changes_table().await?;
        workspace.create_resolves_pending_table().await?;
        workspace.create_branch_merges_pending_table().await?;

        Ok(workspace)
    }

    #[span_fn]
    pub async fn connect(lsc_root: PathBuf) -> Result<Self> {
        let db_uri = Self::db_uri(&lsc_root);

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
            "CREATE TABLE `{}`(key VARCHAR(255), value VARCHAR(8192), unique(key));",
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
            "CREATE TABLE `{}`(relative_path TEXT NOT NULL PRIMARY KEY, change_type INTEGER);",
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
            "CREATE TABLE `{}`(relative_path VARCHAR(512) NOT NULL PRIMARY KEY, base_commit_id VARCHAR(255), theirs_commit_id VARCHAR(255));",
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
            "CREATE TABLE `{}`(name VARCHAR(255) NOT NULL PRIMARY KEY, head VARCHAR(255));",
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
    #[span_fn]
    async fn get_current_branch(&self) -> Result<(String, String)> {
        let sql: &str = &format!("SELECT value FROM `{}` WHERE key = ?;", Self::TABLE_CONFIG);
        let sql = sqlx::query(sql).bind(Self::CONFIG_CURRENT_BRANCH);

        let mut conn = self.sql_connection.lock().await;

        let row = conn
            .fetch_one(sql)
            .await
            .map_other_err("failed to get current branch")?;

        let current_branch: CurrentBranch = serde_json::from_str(row.get("value"))
            .map_other_err("failed to deserialize current branch information")?;

        Ok((current_branch.branch_name, current_branch.commit_id))
    }

    #[span_fn]
    async fn set_current_branch(&self, branch_name: &str, commit_id: &str) -> Result<()> {
        let value = serde_json::to_string(&CurrentBranch {
            branch_name: branch_name.into(),
            commit_id: commit_id.into(),
        })
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

    #[span_fn]
    async fn get_local_changes(&self) -> Result<Vec<LocalChange>> {
        let sql: &str = &format!(
            "SELECT relative_path, change_type FROM {}",
            Self::TABLE_CHANGES
        );

        let mut conn = self.sql_connection.lock().await;

        let rows = conn
            .fetch_all(sql)
            .await
            .map_other_err("failed to set current branch")?;

        drop(conn);

        let mut res = vec![];

        for row in rows {
            let change_type_int: i64 = row.get("change_type");

            res.push(LocalChange::new(
                row.get("relative_path"),
                ChangeType::from_int(change_type_int).unwrap(),
            ));
        }

        Ok(res)
    }

    #[span_fn]
    async fn find_local_change(
        &self,
        canonical_relative_path: &str,
    ) -> Result<Option<LocalChange>> {
        let path = &canonical_relative_path.to_lowercase();

        let sql: &str = &format!(
            "SELECT change_type FROM {} WHERE relative_path = ?;",
            Self::TABLE_CHANGES
        );

        let sql = sqlx::query(sql).bind(path);

        let mut conn = self.sql_connection.lock().await;

        Ok(conn
            .fetch_optional(sql)
            .await
            .map_other_err("failed to fetch local change")?
            .map(|row| {
                let change_type_int: i64 = row.get("change_type");

                LocalChange::new(path, ChangeType::from_int(change_type_int).unwrap())
            }))
    }

    #[span_fn]
    async fn save_local_change(&self, change_spec: &LocalChange) -> Result<()> {
        let sql: &str = &format!("REPLACE INTO {} VALUES(?, ?);", Self::TABLE_CHANGES);
        let sql = sqlx::query(sql)
            .bind(change_spec.relative_path.clone())
            .bind(change_spec.change_type.clone() as i64);

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err(format!(
                "failed to save local change to: {}",
                change_spec.relative_path,
            ))
            .map_other_err("failed to save local change")
            .map(|_| ())
    }

    #[span_fn]
    async fn clear_local_changes(&self, changes: &[LocalChange]) -> Result<()> {
        let mut conn = self.sql_connection.lock().await;
        let mut transaction = conn
            .begin()
            .await
            .map_other_err("failed to begin transaction")?;

        for change in changes {
            let sql: &str = &format!("DELETE from {} where relative_path=?;", Self::TABLE_CHANGES);
            let sql = sqlx::query(sql).bind(&change.relative_path);

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

    #[span_fn]
    async fn read_pending_branch_merges(&self) -> Result<Vec<PendingBranchMerge>> {
        let sql: &str = &format!(
            "SELECT name, head FROM {};",
            Self::TABLE_BRANCH_MERGES_PENDING
        );

        let mut conn = self.sql_connection.lock().await;

        Ok(conn
            .fetch_all(sql)
            .await
            .map_other_err("failed to read pending branch merges")?
            .into_iter()
            .map(|row| PendingBranchMerge {
                name: row.get("name"),
                head: row.get("head"),
            })
            .collect())
    }

    #[span_fn]
    async fn clear_pending_branch_merges(&self) -> Result<()> {
        let sql: &str = &format!("DELETE from {};", Self::TABLE_BRANCH_MERGES_PENDING);

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to clear pending branch merges")
            .map(|_| ())
    }

    #[span_fn]
    async fn save_pending_branch_merge(&self, merge_spec: &PendingBranchMerge) -> Result<()> {
        let sql: &str = &format!(
            "INSERT OR REPLACE into {} VALUES(?,?);",
            Self::TABLE_BRANCH_MERGES_PENDING
        );
        let sql = sqlx::query(sql)
            .bind(merge_spec.name.clone())
            .bind(merge_spec.head.clone());

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to save pending branch merge")
            .map(|_| ())
    }

    #[span_fn]
    async fn save_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()> {
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

    #[span_fn]
    async fn clear_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()> {
        let sql: &str = &format!(
            "DELETE from {}
             WHERE relative_path=?;",
            Self::TABLE_RESOLVES_PENDING
        );
        let sql = sqlx::query(sql).bind(resolve_pending.relative_path.clone());

        let mut conn = self.sql_connection.lock().await;

        conn.execute(sql)
            .await
            .map_other_err("failed to save resolve pending")
            .map(|_| ())
    }

    #[span_fn]
    async fn find_resolve_pending(
        &self,
        canonical_relative_path: &str,
    ) -> Result<Option<ResolvePending>> {
        let sql: &str = &format!(
            "SELECT base_commit_id, theirs_commit_id 
             FROM {}
             WHERE relative_path = ?;",
            Self::TABLE_RESOLVES_PENDING
        );
        let sql = sqlx::query(sql).bind(canonical_relative_path);

        let mut conn = self.sql_connection.lock().await;

        Ok(conn
            .fetch_optional(sql)
            .await
            .map_other_err("failed to find resolve pending")?
            .map(|row| {
                ResolvePending::new(
                    String::from(canonical_relative_path),
                    row.get("base_commit_id"),
                    row.get("theirs_commit_id"),
                )
            }))
    }

    #[span_fn]
    async fn read_resolves_pending(&self) -> Result<Vec<ResolvePending>> {
        let sql: &str = &format!(
            "SELECT relative_path, base_commit_id, theirs_commit_id 
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
                    row.get("relative_path"),
                    row.get("base_commit_id"),
                    row.get("theirs_commit_id"),
                )
            })
            .collect())
    }
}
