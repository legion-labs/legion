use crate::*;
use std::collections::BTreeSet;

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
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let current_branch = read_current_branch(&workspace_root)?;
    let mut repo_branch = connection.query().read_branch(&current_branch.name).await?;
    repo_branch.parent.clear();

    let locks_in_old_domain = read_locks(&connection, &repo_branch.lock_domain_id)?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();

    let descendants = find_branch_descendants(connection.query(), &current_branch.name).await?;

    if let Err(e) = connection.query().update_branch(&repo_branch).await {
        return Err(format!(
            "Error saving {} to clear its parent: {}",
            repo_branch.name, e
        ));
    }

    let mut errors = Vec::new();

    for branch_name in &descendants {
        match connection.query().read_branch(branch_name).await {
            Ok(mut branch) => {
                branch.lock_domain_id = lock_domain_id.clone();
                if let Err(e) = connection.query().update_branch(&branch).await {
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
            if let Err(e) = save_new_lock(&connection, &new_lock) {
                errors.push(format!(
                    "Error writing lock in new domain for {}: {}",
                    lock.relative_path, e
                ));
            }
            if let Err(e) = clear_lock(&connection, &lock.lock_domain_id, &lock.relative_path) {
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
