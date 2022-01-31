use std::path::Path;

use anyhow::{Context, Result};

use crate::{make_canonical_relative_path, Lock, Workspace};

pub async fn verify_empty_lock_domain(workspace: &Workspace, lock_domain_id: &str) -> Result<()> {
    if workspace
        .index_backend
        .count_locks_in_domain(lock_domain_id)
        .await?
        > 0
    {
        anyhow::bail!("lock domain not empty: {}", lock_domain_id);
    }

    Ok(())
}

pub async fn lock_file_command(path_specified: impl AsRef<Path>) -> Result<()> {
    let workspace = Workspace::find(path_specified.as_ref()).await?;
    let (branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let repo_branch = workspace.index_backend.read_branch(&branch_name).await?;
    let lock = Lock {
        relative_path: make_canonical_relative_path(&workspace.root, path_specified)?,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace.registration.id,
        branch_name: repo_branch.name,
    };
    workspace
        .index_backend
        .insert_lock(&lock)
        .await
        .map_err(Into::into)
}

pub async fn unlock_file_command(path_specified: impl AsRef<Path>) -> Result<()> {
    let workspace = Workspace::find(path_specified.as_ref()).await?;
    let (branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let repo_branch = workspace.index_backend.read_branch(&branch_name).await?;
    let relative_path = make_canonical_relative_path(&workspace.root, path_specified)?;
    workspace
        .index_backend
        .clear_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
        .map_err(Into::into)
}

pub async fn list_locks_command() -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let (branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let repo_branch = workspace.index_backend.read_branch(&branch_name).await?;
    let locks = workspace
        .index_backend
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

pub async fn assert_not_locked(workspace: &Workspace, path_specified: &Path) -> Result<()> {
    let (current_branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let repo_branch = workspace
        .index_backend
        .read_branch(&current_branch_name)
        .await?;
    let relative_path = make_canonical_relative_path(&workspace.root, path_specified)?;

    match workspace
        .index_backend
        .find_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
        .context(format!(
            "error validating that {} is lock-free",
            relative_path,
        ))? {
        Some(lock) => {
            if lock.branch_name == current_branch_name
                && lock.workspace_id == workspace.registration.id
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
