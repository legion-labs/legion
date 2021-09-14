use crate::*;
use std::collections::BTreeSet;

pub async fn attach_branch_command(parent_branch_name: &str) -> Result<(), String> {
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
        return Err(format!(
            "Can't attach branch {} to {}: branch {} already has {} for parent",
            current_branch_name, parent_branch_name, current_branch_name, repo_branch.parent
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

    let mut errors = Vec::new();
    let locks_to_move = query
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;
    for lock in &locks_to_move {
        //validate first before making the change
        if locks_parent_domain.contains(&lock.relative_path) {
            errors.push(format!("Lock domains conflict on {}", lock.relative_path));
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    //looks good, let's roll

    for lock in &locks_to_move {
        let mut new_lock = lock.clone();
        new_lock.lock_domain_id = parent_branch.lock_domain_id.clone();
        if let Err(e) = query.insert_lock(&new_lock).await {
            errors.push(format!(
                "Error creating new lock for {}: {}",
                new_lock.relative_path, e
            ));
        }
        if let Err(e) = query
            .clear_lock(&lock.lock_domain_id, &lock.relative_path)
            .await
        {
            errors.push(format!(
                "Error clearing old lock for {}: {}",
                new_lock.relative_path, e
            ));
        }
        println!("Moved lock for {}", lock.relative_path);
    }

    repo_branch.parent = parent_branch.name;
    if let Err(e) = query.update_branch(&repo_branch).await {
        return Err(format!(
            "Error saving {} to set its parent: {}",
            repo_branch.name, e
        ));
    }

    match query
        .find_branches_in_lock_domain(&repo_branch.lock_domain_id)
        .await
    {
        Ok(branches) => {
            for mut branch in branches {
                branch.lock_domain_id = parent_branch.lock_domain_id.clone();
                if let Err(e) = query.update_branch(&branch).await {
                    errors.push(format!("Error saving branch {}: {}", branch.name, e));
                } else {
                    println!("Updated branch {}", branch.name);
                }
            }
        }
        Err(e) => {
            errors.push(format!(
                "Error listing branches in domain {}: {}",
                repo_branch.lock_domain_id, e
            ));
        }
    }

    if let Err(e) = verify_empty_lock_domain(query, &repo_branch.lock_domain_id).await {
        errors.push(e);
    } else {
        println!("Deleted lock domain {}", repo_branch.lock_domain_id);
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}
