use std::collections::BTreeSet;

use crate::{
    connect_to_server, find_workspace_root, read_current_branch, read_workspace_spec,
    LocalWorkspaceConnection, RepositoryQuery,
};

// find_branch_descendants includes the branch itself
async fn find_branch_descendants(
    query: &dyn RepositoryQuery,
    root_branch_name: &str,
) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    set.insert(String::from(root_branch_name));
    let branches = query.read_branches().await?;
    let mut keep_going = true;
    while keep_going {
        keep_going = false;
        for branch in &branches {
            if set.contains(&branch.parent) && !set.contains(&branch.name) {
                set.insert(branch.name.clone());
                keep_going = true;
            }
        }
    }
    Ok(set)
}

pub async fn detach_branch_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let (current_branch_name, _current_commit) =
        read_current_branch(workspace_connection.sql()).await?;
    let mut repo_branch = query.read_branch(&current_branch_name).await?;
    repo_branch.parent.clear();

    let locks_in_old_domain = query
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();

    let descendants = find_branch_descendants(query, &current_branch_name).await?;

    if let Err(e) = query.update_branch(&repo_branch).await {
        return Err(format!(
            "Error saving {} to clear its parent: {}",
            repo_branch.name, e
        ));
    }

    let mut errors = Vec::new();

    for branch_name in &descendants {
        match query.read_branch(branch_name).await {
            Ok(mut branch) => {
                branch.lock_domain_id = lock_domain_id.clone();
                if let Err(e) = query.update_branch(&branch).await {
                    errors.push(format!("Error updating branch {}: {}", branch_name, e));
                } else {
                    println!("updated branch {}", branch_name);
                }
            }
            Err(e) => {
                errors.push(format!("Error reading branch {}: {}", &branch_name, e));
            }
        }
    }

    for lock in locks_in_old_domain {
        if descendants.contains(&lock.branch_name) {
            let mut new_lock = lock.clone();
            new_lock.lock_domain_id = lock_domain_id.clone();
            println!("moving lock for {}", lock.relative_path);
            if let Err(e) = query.insert_lock(&new_lock).await {
                errors.push(format!(
                    "Error writing lock in new domain for {}: {}",
                    lock.relative_path, e
                ));
            }
            if let Err(e) = query
                .clear_lock(&lock.lock_domain_id, &lock.relative_path)
                .await
            {
                errors.push(format!(
                    "Error clearning lock from old domain for {}: {}",
                    lock.relative_path, e
                ));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}
