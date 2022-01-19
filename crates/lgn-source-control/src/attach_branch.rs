use std::collections::BTreeSet;

use anyhow::{Context, Result};

use crate::{verify_empty_lock_domain, Workspace};

#[lgn_tracing::span_fn]
pub async fn attach_branch_command(parent_branch_name: &str) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;

    let (current_branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let mut repo_branch = workspace
        .index_backend
        .read_branch(&current_branch_name)
        .await?;

    if !repo_branch.parent.is_empty() {
        return Err(anyhow::format_err!(
            "can't attach branch `{}` to `{}`: branch already has `{}` for parent",
            current_branch_name,
            parent_branch_name,
            repo_branch.parent
        ));
    }

    let parent_branch = workspace
        .index_backend
        .read_branch(parent_branch_name)
        .await?;
    let mut locks_parent_domain = BTreeSet::new();

    for lock in workspace
        .index_backend
        .find_locks_in_domain(&parent_branch.lock_domain_id)
        .await?
    {
        locks_parent_domain.insert(lock.relative_path);
    }

    let mut conflicting_paths = Vec::new();
    let locks_to_move = workspace
        .index_backend
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;

    for lock in &locks_to_move {
        //validate first before making the change
        if locks_parent_domain.contains(&lock.relative_path) {
            conflicting_paths.push(lock.relative_path.clone());
        }
    }

    if !conflicting_paths.is_empty() {
        return Err(anyhow::format_err!(
            "lock domains conflicts: {}",
            conflicting_paths.join(", ")
        ));
    }

    drop(conflicting_paths);

    //looks good, let's roll

    let mut failed_new_locks = vec![];
    let mut failed_old_locks = vec![];

    for lock in &locks_to_move {
        let mut new_lock = lock.clone();
        new_lock.lock_domain_id = parent_branch.lock_domain_id.clone();

        if let Err(e) = workspace.index_backend.insert_lock(&new_lock).await {
            failed_new_locks.push(format!("{}: {}", new_lock.relative_path, e));
        }

        if let Err(e) = workspace
            .index_backend
            .clear_lock(&lock.lock_domain_id, &lock.relative_path)
            .await
        {
            failed_old_locks.push(format!("{}: {}", lock.relative_path.clone(), e));
        }

        println!("Moved lock for {}", lock.relative_path);
    }

    repo_branch.parent = parent_branch.name;

    workspace
        .index_backend
        .update_branch(&repo_branch)
        .await
        .context(format!("error saving branch {}", repo_branch.name))?;

    let branches = workspace
        .index_backend
        .find_branches_in_lock_domain(&repo_branch.lock_domain_id)
        .await
        .context(format!(
            "error finding branches in lock domain {}",
            repo_branch.lock_domain_id
        ))?;

    for mut branch in branches {
        branch.lock_domain_id = parent_branch.lock_domain_id.clone();

        workspace
            .index_backend
            .update_branch(&branch)
            .await
            .context(format!("error saving branch {}", branch.name))?;

        println!("Updated branch {}", branch.name);
    }

    verify_empty_lock_domain(&workspace, &repo_branch.lock_domain_id).await?;

    println!("Deleted lock domain {}", repo_branch.lock_domain_id);

    // Not sure why this is so late in the process. We should probably revisit
    // the logic and error reporting at some point.

    if !failed_new_locks.is_empty() {
        return Err(anyhow::format_err!(
            "failed to create new locks: {}",
            failed_new_locks.join(", ")
        ));
    }

    if !failed_old_locks.is_empty() {
        return Err(anyhow::format_err!(
            "failed to delete old locks: {}",
            failed_old_locks.join(", ")
        ));
    }

    Ok(())
}
