use crate::*;
use std::collections::BTreeSet;

fn find_branches_in_lock_domain(
    connection: &RepositoryConnection,
    lock_domain_id: &str,
) -> Result<Vec<Branch>, String> {
    let mut res = Vec::new();
    for branch in read_branches(connection)? {
        if branch.lock_domain_id == lock_domain_id {
            res.push(branch);
        }
    }
    Ok(res)
}

pub async fn attach_branch_command(parent_branch_name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let mut connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let current_branch = read_current_branch(&workspace_root)?;
    let mut repo_branch = query.read_branch(&current_branch.name).await?;
    if !repo_branch.parent.is_empty() {
        return Err(format!(
            "Can't attach branch {} to {}: branch {} already has {} for parent",
            current_branch.name, parent_branch_name, current_branch.name, repo_branch.parent
        ));
    }

    let parent_branch = query.read_branch(parent_branch_name).await?;
    let mut locks_parent_domain = BTreeSet::new();
    for lock in read_locks(&connection, &parent_branch.lock_domain_id)? {
        locks_parent_domain.insert(lock.relative_path);
    }

    let mut errors = Vec::new();
    let locks_to_move = read_locks(&connection, &repo_branch.lock_domain_id)?;
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
        if let Err(e) = save_new_lock(&connection, &new_lock) {
            errors.push(format!(
                "Error creating new lock for {}: {}",
                new_lock.relative_path, e
            ));
        }
        if let Err(e) = clear_lock(&connection, &lock.lock_domain_id, &lock.relative_path) {
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

    match find_branches_in_lock_domain(&connection, &repo_branch.lock_domain_id) {
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

    if let Err(e) = verify_empty_lock_domain(&mut connection, &repo_branch.lock_domain_id) {
        errors.push(e);
    } else {
        println!("Deleted lock domain {}", repo_branch.lock_domain_id);
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}
