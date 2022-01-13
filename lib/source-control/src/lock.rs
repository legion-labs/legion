use std::path::Path;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    connect_to_server, find_workspace_root, make_canonical_relative_path, read_current_branch,
    read_workspace_spec, IndexBackend, LocalWorkspaceConnection,
};

#[derive(Debug, Clone)]
pub struct Lock {
    pub relative_path: String, /* needs to have a stable representation across platforms because
                                * it seeds the hash */
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

impl From<Lock> for lgn_source_control_proto::Lock {
    fn from(lock: Lock) -> Self {
        Self {
            relative_path: lock.relative_path,
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        }
    }
}

impl From<lgn_source_control_proto::Lock> for Lock {
    fn from(lock: lgn_source_control_proto::Lock) -> Self {
        Self {
            relative_path: lock.relative_path,
            lock_domain_id: lock.lock_domain_id,
            workspace_id: lock.workspace_id,
            branch_name: lock.branch_name,
        }
    }
}

pub async fn verify_empty_lock_domain(
    query: &dyn IndexBackend,
    lock_domain_id: &str,
) -> Result<()> {
    if query.count_locks_in_domain(lock_domain_id).await? > 0 {
        anyhow::bail!("lock domain not empty: {}", lock_domain_id);
    }

    Ok(())
}

#[span_fn]
pub async fn lock_file_command(path_specified: &Path) -> Result<()> {
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.index_backend();
    let repo_branch = query.read_branch(&branch_name).await?;
    let lock = Lock {
        relative_path: make_canonical_relative_path(&workspace_root, path_specified)?,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace_spec.registration.id,
        branch_name: repo_branch.name,
    };
    query.insert_lock(&lock).await.map_err(Into::into)
}

#[span_fn]
pub async fn unlock_file_command(path_specified: &Path) -> Result<()> {
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.index_backend();
    let repo_branch = query.read_branch(&branch_name).await?;
    let relative_path = make_canonical_relative_path(&workspace_root, path_specified)?;
    query
        .clear_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
        .map_err(Into::into)
}

#[span_fn]
pub async fn list_locks_command() -> Result<()> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.index_backend();
    let repo_branch = query.read_branch(&branch_name).await?;
    let locks = query
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;
    if locks.is_empty() {
        println!("no locks found in domain {}", &repo_branch.lock_domain_id);
    }
    for lock in locks {
        println!(
            "{} in branch {} owned by workspace {}",
            &lock.relative_path, &lock.branch_name, &lock.workspace_id
        );
    }
    Ok(())
}

#[span_fn]
pub async fn assert_not_locked(
    query: &dyn IndexBackend,
    workspace_root: &Path,
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    path_specified: &Path,
) -> Result<()> {
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let (current_branch_name, _current_commit) = read_current_branch(transaction).await?;
    let repo_branch = query.read_branch(&current_branch_name).await?;
    let relative_path = make_canonical_relative_path(workspace_root, path_specified)?;

    match query
        .find_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
        .context(format!(
            "error validating that {} is lock-free",
            relative_path,
        ))? {
        Some(lock) => {
            if lock.branch_name == current_branch_name
                && lock.workspace_id == workspace_spec.registration.id
            {
                Ok(()) //locked by this workspace on this branch - all good
            } else {
                anyhow::bail!(
                    "file {} locked in branch {}, owned by workspace {}",
                    lock.relative_path,
                    lock.branch_name,
                    lock.workspace_id
                )
            }
        }
        None => Ok(()),
    }
}
