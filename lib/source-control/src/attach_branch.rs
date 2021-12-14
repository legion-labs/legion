use anyhow::{Context, Result};
use std::collections::BTreeSet;

use crate::{
    connect_to_server, find_workspace_root, read_current_branch, read_workspace_spec, trace_scope,
    verify_empty_lock_domain, LocalWorkspaceConnection,
};

pub async fn attach_branch_command(parent_branch_name: &str) -> Result<()> {
    trace_scope!();

    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let (current_branch_name, _current_commit) =
        read_current_branch(workspace_connection.sql()).await?;
    let mut repo_branch = query.read_branch(&current_branch_name).await?;

    if !repo_branch.parent.is_empty() {
        return Err(anyhow::format_err!(
            "can't attach branch `{}` to `{}`: branch already has `{}` for parent",
            current_branch_name,
            parent_branch_name,
            repo_branch.parent
        ));
    }

    let parent_branch = query.read_branch(parent_branch_name).await?;
    let mut locks_parent_domain = BTreeSet::new();

    for lock in query
        .find_locks_in_domain(&parent_branch.lock_domain_id)
        .await?
    {
        locks_parent_domain.insert(lock.relative_path);
    }

    let mut conflicting_paths = Vec::new();
    let locks_to_move = query
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

        if let Err(e) = query.insert_lock(&new_lock).await {
            failed_new_locks.push(format!("{}: {}", new_lock.relative_path, e.to_string()));
        }

        if let Err(e) = query
            .clear_lock(&lock.lock_domain_id, &lock.relative_path)
            .await
        {
            failed_old_locks.push(format!("{}: {}", lock.relative_path.clone(), e.to_string()));
        }

        println!("Moved lock for {}", lock.relative_path);
    }

    repo_branch.parent = parent_branch.name;

    query
        .update_branch(&repo_branch)
        .await
        .context(format!("error saving branch {}", repo_branch.name))?;

    let branches = query
        .find_branches_in_lock_domain(&repo_branch.lock_domain_id)
        .await
        .context(format!(
            "error finding branches in lock domain {}",
            repo_branch.lock_domain_id
        ))?;

    for mut branch in branches {
        branch.lock_domain_id = parent_branch.lock_domain_id.clone();

        query
            .update_branch(&branch)
            .await
            .context(format!("error saving branch {}", branch.name))?;

        println!("Updated branch {}", branch.name);
    }

    verify_empty_lock_domain(query, &repo_branch.lock_domain_id).await?;

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
