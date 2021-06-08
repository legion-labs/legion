use crate::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

// find_branch_descendants includes the branch itself
fn find_branch_descendants(
    repo: &Path,
    root_branch_name: &str,
) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    set.insert(String::from(root_branch_name));
    let branches = read_branches(repo)?;
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
    let repo = &workspace_spec.repository;
    let current_branch = read_current_branch(&workspace_root)?;
    let mut repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    repo_branch.parent.clear();

    let locks_in_old_domain = read_locks(repo, &repo_branch.lock_domain_id)?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = fs::create_dir_all(repo.join(format!("lock_domains/{}", lock_domain_id))) {
        return Err(format!("Error creating locks directory: {}", e));
    }

    let descendants = find_branch_descendants(repo, &current_branch.name)?;

    if let Err(e) = save_branch_to_repo(repo, &repo_branch) {
        return Err(format!(
            "Error saving {} to clear its parent: {}",
            repo_branch.name, e
        ));
    }

    let mut errors = Vec::new();

    for branch_name in &descendants {
        match read_branch_from_repo(repo, branch_name) {
            Ok(mut branch) => {
                branch.lock_domain_id = lock_domain_id.clone();
                if let Err(e) = save_branch_to_repo(repo, &branch) {
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
            if let Err(e) = save_lock(repo, &new_lock) {
                errors.push(format!(
                    "Error writing lock in new domain for {}: {}",
                    lock.relative_path, e
                ));
            }
            if let Err(e) = clear_lock(repo, &lock.lock_domain_id, &lock.relative_path) {
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
