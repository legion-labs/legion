use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    assert_not_locked, connect_to_server, find_file_hash_at_commit, find_workspace_root,
    make_canonical_relative_path, make_file_read_only, make_path_absolute, read_current_branch,
    read_workspace_spec, sql::execute_sql, LocalWorkspaceConnection, RepositoryConnection,
    RepositoryQuery,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    Edit = 1,
    Add = 2,
    Delete = 3,
}

impl ChangeType {
    pub fn from_int(i: i64) -> Result<Self> {
        match i {
            1 => Ok(Self::Edit),
            2 => Ok(Self::Add),
            3 => Ok(Self::Delete),
            _ => anyhow::bail!("invalid change type {}", i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalChange {
    pub relative_path: String,
    pub change_type: ChangeType,
}

impl LocalChange {
    pub fn new(canonical_relative_path: &str, change_type: ChangeType) -> Self {
        Self {
            relative_path: canonical_relative_path.to_lowercase(),
            change_type,
        }
    }
}

pub async fn init_local_changes_database(connection: &mut LocalWorkspaceConnection) -> Result<()> {
    let sql_connection = connection.sql();
    let sql = "CREATE TABLE changes(relative_path TEXT NOT NULL PRIMARY KEY, change_type INTEGER);";

    execute_sql(sql_connection, sql)
        .await
        .context("failed to create changes table")
}

pub async fn save_local_change(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    change_spec: &LocalChange,
) -> Result<()> {
    sqlx::query("REPLACE INTO changes VALUES(?, ?);")
        .bind(change_spec.relative_path.clone())
        .bind(change_spec.change_type.clone() as i64)
        .execute(transaction)
        .await
        .context(format!(
            "failed to save local change to: {}",
            change_spec.relative_path,
        ))?;

    Ok(())
}

pub async fn find_local_change(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    canonical_relative_path: &str,
) -> Result<Option<LocalChange>> {
    let path = canonical_relative_path.to_lowercase();

    Ok(sqlx::query(
        "SELECT change_type
             FROM changes
             WHERE relative_path = ?;",
    )
    .bind(path.clone())
    .fetch_optional(transaction)
    .await
    .context("error fetching local change")?
    .map(|row| {
        let change_type_int: i64 = row.get("change_type");

        LocalChange::new(&path, ChangeType::from_int(change_type_int).unwrap())
    }))
}

#[span_fn]
pub async fn read_local_changes(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<LocalChange>> {
    let rows = sqlx::query(
        "SELECT relative_path, change_type
             FROM changes",
    )
    .fetch_all(transaction)
    .await
    .context("error reading local changes")?;

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
pub async fn find_local_changes_command() -> Result<Vec<LocalChange>> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    read_local_changes(&mut workspace_transaction).await
}

pub async fn clear_local_change(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    change: &LocalChange,
) -> Result<()> {
    sqlx::query("DELETE from changes where relative_path=?;")
        .bind(change.relative_path.clone())
        .execute(workspace_transaction)
        .await
        .context("error clearing local change")?;

    Ok(())
}

pub async fn clear_local_changes(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    local_changes: &[LocalChange],
) {
    for change in local_changes {
        if let Err(e) = clear_local_change(transaction, change).await {
            println!(
                "Error clearing local change {}: {}",
                change.relative_path, e
            );
        }
    }
}

pub async fn track_new_file(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    repo_connection: &RepositoryConnection,
    path_specified: &Path,
) -> Result<()> {
    let abs_path = make_path_absolute(path_specified);

    if !abs_path.exists() {
        anyhow::bail!("Error: file {} not found", abs_path.display());
    }
    //todo: make sure the file does not exist in the current tree hierarchy

    let relative_path = make_canonical_relative_path(workspace_root, &abs_path)?;
    if let Some(change) = find_local_change(workspace_transaction, &relative_path)
        .await
        .context("error searching in local changes")?
    {
        anyhow::bail!(
            "{} already tracked for {:?}",
            change.relative_path,
            change.change_type
        );
    }

    let (_branch_name, current_commit) = read_current_branch(workspace_transaction).await?;

    if let Some(_hash) =
        find_file_hash_at_commit(repo_connection, Path::new(&relative_path), &current_commit)
            .await?
    {
        anyhow::bail!("file already exists in tree");
    }

    assert_not_locked(
        repo_connection.query(),
        workspace_root,
        workspace_transaction,
        &abs_path,
    )
    .await?;
    let local_change = LocalChange::new(&relative_path, ChangeType::Add);

    save_local_change(workspace_transaction, &local_change).await
}

#[span_fn]
pub async fn track_new_file_command(path_specified: &Path) -> Result<()> {
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    track_new_file(
        &workspace_root,
        &mut workspace_transaction,
        &connection,
        path_specified,
    )
    .await?;

    workspace_transaction
        .commit()
        .await
        .context("error committing transaction for track_new_file_command")
}

pub async fn edit_file(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    query: &dyn RepositoryQuery,
    path_specified: &Path,
) -> Result<()> {
    let abs_path = make_path_absolute(path_specified);
    fs::metadata(&abs_path)
        .context(format!("error reading metadata for {}", abs_path.display()))?;

    //todo: make sure file is tracked by finding it in the current tree hierarchy
    assert_not_locked(query, workspace_root, workspace_transaction, &abs_path).await?;

    let relative_path = make_canonical_relative_path(workspace_root, &abs_path)?;

    if let Some(change) = find_local_change(workspace_transaction, &relative_path)
        .await
        .context("error searching in local changes")?
    {
        anyhow::bail!(
            "{} already tracked for {:?}",
            change.relative_path,
            change.change_type
        );
    }

    let local_change = LocalChange::new(&relative_path, ChangeType::Edit);
    save_local_change(workspace_transaction, &local_change).await?;
    make_file_read_only(&abs_path, false)
}

#[span_fn]
pub async fn edit_file_command(path_specified: &Path) -> Result<()> {
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;

    edit_file(
        &workspace_root,
        &mut workspace_transaction,
        connection.query(),
        path_specified,
    )
    .await?;

    workspace_transaction
        .commit()
        .await
        .context("error in transaction commit for edit_file_command")
}
