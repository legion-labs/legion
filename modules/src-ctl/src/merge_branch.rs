use crate::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn find_latest_common_ancestor(
    sequence_branch_one: &[Commit],
    set_branch_two: &BTreeSet<String>,
) -> Option<String> {
    // if the times are reliable we can cut short this search
    for c in sequence_branch_one {
        if set_branch_two.contains(&c.id) {
            return Some(c.id.clone());
        }
    }
    None
}

fn change_file_to(
    repo: &Path,
    relative_path: &Path,
    workspace_root: &Path,
    hash_to_sync: &str,
) -> Result<String, String> {
    let local_path = workspace_root.join(relative_path);
    if local_path.exists() {
        let local_hash = compute_file_hash(&local_path)?;
        if local_hash == hash_to_sync {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if hash_to_sync.is_empty() {
            delete_file_command(&local_path)?;
            return Ok(format!("Deleted {}", local_path.display()));
        }
        edit_file_command(&local_path)?;
        if let Err(e) = download_blob(&repo, &local_path, &hash_to_sync) {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        return Ok(format!("Updated {}", local_path.display()));
    } else {
        //no local file
        if hash_to_sync.is_empty() {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if let Err(e) = download_blob(&repo, &local_path, &hash_to_sync) {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        track_new_file(&local_path)?;
        return Ok(format!("Added {}", local_path.display()));
    }
}

pub fn merge_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let branch_to_merge = read_branch_from_repo(&repo, &name)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let mut latest_branch = read_branch_from_repo(&repo, &current_branch.name)?;

    let branch_commits = find_branch_commits(&repo, &branch_to_merge)?;
    let mut branch_commit_ids_set: BTreeSet<String> = BTreeSet::new();
    for c in &branch_commits {
        branch_commit_ids_set.insert(c.id.clone());
    }
    if branch_commit_ids_set.contains(&latest_branch.head) {
        //fast forward case
        latest_branch.head = branch_to_merge.head;
        save_current_branch(&workspace_root, &latest_branch)?;
        save_branch_to_repo(&repo, &latest_branch)?;
        println!("Fast-forward merge: branch updated, synching");
        return sync_command();
    }

    if current_branch.head != latest_branch.head {
        return Err(String::from(
            "Workspace not up to date, sync to latest before merge",
        ));
    }

    let mut errors: Vec<String> = Vec::new();
    let latest_commits = find_branch_commits(&repo, &latest_branch)?;
    if let Some(common_ancestor_id) =
        find_latest_common_ancestor(&latest_commits, &branch_commit_ids_set)
    {
        let mut modified_in_current: BTreeMap<PathBuf, String> = BTreeMap::new();
        for commit in &latest_commits {
            if commit.id == common_ancestor_id {
                break;
            }
            for change in &commit.changes {
                modified_in_current
                    .entry(change.relative_path.clone())
                    .or_insert_with(|| change.hash.clone());
            }
        }

        let mut to_update: BTreeMap<PathBuf, String> = BTreeMap::new();
        for commit in &branch_commits {
            if commit.id == common_ancestor_id {
                break;
            }
            for change in &commit.changes {
                to_update
                    .entry(change.relative_path.clone())
                    .or_insert_with(|| change.hash.clone());
            }
        }

        for (path, hash) in to_update.iter() {
            if modified_in_current.contains_key(path) {
                //todo: support conflicts
                return Err(format!(
                    "merge aborted, conflict found with {}",
                    path.display()
                ));
            }
            match change_file_to(&repo, &path, &workspace_root, &hash) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    } else {
        return Err(String::from(
            "Error finding common ancestor for branch merge",
        ));
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    println!("merge completed, ready to commit");
    Ok(())
}
