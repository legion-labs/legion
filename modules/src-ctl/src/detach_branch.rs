use crate::*;
use std::collections::BTreeSet;

// find_branch_descendants includes the branch itself
fn find_branch_descendants(
    connection: &mut RepositoryConnection,
    root_branch_name: &str,
) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    set.insert(String::from(root_branch_name));
    let branches = read_branches(connection)?;
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

pub fn detach_branch_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let current_branch = read_current_branch(&workspace_root)?;
    let mut repo_branch = read_branch_from_repo(&mut connection, &current_branch.name)?;
    repo_branch.parent.clear();

    let locks_in_old_domain = read_locks(&mut connection, &repo_branch.lock_domain_id)?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();

    let descendants = find_branch_descendants(&mut connection, &current_branch.name)?;

    if let Err(e) = save_branch_to_repo(&mut connection, &repo_branch) {
        return Err(format!(
            "Error saving {} to clear its parent: {}",
            repo_branch.name, e
        ));
    }

    let mut errors = Vec::new();

    for branch_name in &descendants {
        match read_branch_from_repo(&mut connection, branch_name) {
            Ok(mut branch) => {
                branch.lock_domain_id = lock_domain_id.clone();
                if let Err(e) = save_branch_to_repo(&mut connection, &branch) {
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
            if let Err(e) = save_new_lock(&mut connection, &new_lock) {
                errors.push(format!(
                    "Error writing lock in new domain for {}: {}",
                    lock.relative_path, e
                ));
            }
            if let Err(e) = clear_lock(&mut connection, &lock.lock_domain_id, &lock.relative_path) {
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
