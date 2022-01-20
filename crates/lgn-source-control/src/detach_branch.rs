use std::collections::BTreeSet;

use anyhow::{Context, Result};

use crate::Workspace;

// find_branch_descendants includes the branch itself
async fn find_branch_descendants(
    workspace: &Workspace,
    root_branch_name: &str,
) -> Result<BTreeSet<String>> {
    let mut set = BTreeSet::new();
    set.insert(String::from(root_branch_name));
    let branches = workspace.index_backend.read_branches().await?;
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

pub async fn detach_branch_command() -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let (current_branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let mut repo_branch = workspace
        .index_backend
        .read_branch(&current_branch_name)
        .await?;
    repo_branch.parent.clear();

    let locks_in_old_domain = workspace
        .index_backend
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();

    let descendants = find_branch_descendants(&workspace, &current_branch_name).await?;

    workspace
        .index_backend
        .update_branch(&repo_branch)
        .await
        .context(format!(
            "error saving branch `{}` to clear its parent",
            repo_branch.name
        ))?;

    let mut errors = Vec::new();

    for branch_name in &descendants {
        match workspace.index_backend.read_branch(branch_name).await {
            Ok(mut branch) => {
                branch.lock_domain_id = lock_domain_id.clone();
                if let Err(e) = workspace.index_backend.update_branch(&branch).await {
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
            if let Err(e) = workspace.index_backend.insert_lock(&new_lock).await {
                errors.push(format!(
                    "Error writing lock in new domain for {}: {}",
                    lock.relative_path, e
                ));
            }
            if let Err(e) = workspace
                .index_backend
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
        anyhow::bail!("{}", errors.join("\n"));
    }

    Ok(())
}
