use crate::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn find_commits_to_sync(
    repo: &Path,
    latest_commit_id: &str,
    current_commit_id: &str,
) -> Result<Vec<Commit>, String> {
    let mut commits = Vec::new();
    let mut c = read_commit(&repo, &latest_commit_id)?;
    while c.id != current_commit_id {
        commits.push(c.clone());
        let parent_id = &c.parents[0]; //first parent is assumed to be branch trunk
        c = read_commit(&repo, &parent_id)?;
    }
    Ok(commits)
}

fn compute_file_hash(p: &Path) -> Result<String, String> {
    //todo: protect against missing local file
    let contents = read_bin_file(p)?;
    let hash = format!("{:X}", Sha256::digest(&contents));
    Ok(hash)
}

fn sync_file(repo: &Path, local_path: &Path, hash_to_sync: &str) -> Result<String, String> {
    let local_hash = compute_file_hash(&local_path)?;
    if local_hash == hash_to_sync{
        return Ok(format!("Verified {}", local_path.display()));
    }
    match fs::metadata(&local_path) {
        Ok(meta) => {
            let mut permissions = meta.permissions();
            if !permissions.readonly() {
                return Err(format!(
                    "Error: local file {} is writable. Skipping sync for this file.",
                    local_path.display()
                ));
            }

            permissions.set_readonly(false);
            if let Err(e) = fs::set_permissions(&local_path, permissions) {
                return Err(format!(
                    "Error making file {} writable: {}",
                    local_path.display(),
                    e
                ));
            }

            if hash_to_sync.is_empty() {
                if let Err(e) = fs::remove_file(&local_path) {
                    return Err(format!("Error deleting {}: {}", local_path.display(), e));
                } else {
                    return Ok(format!("Deleted {}", local_path.display()));
                }
            }
            if let Err(e) = download_blob(&repo, &local_path, &hash_to_sync) {
                return Err(format!(
                    "Error downloading {} {}: {}",
                    local_path.display(),
                    &hash_to_sync,
                    e
                ));
            }
            return Ok(format!("Updated {}", local_path.display()));
        }
        Err(e) => {
            return Err(format!(
                "Error reading metadata for {}: {}",
                local_path.display(),
                e
            ));
        }
    }
}

pub fn sync_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let mut workspace_branch = read_current_branch(&workspace_root)?;
    let repo_branch = read_branch_from_repo(&workspace_spec.repository, &workspace_branch.name)?;
    let commits = find_commits_to_sync(
        &workspace_spec.repository,
        &repo_branch.head,
        &workspace_branch.head,
    )?;
    let mut to_download: BTreeMap<PathBuf, String> = BTreeMap::new();
    for commit in commits {
        for change in commit.changes {
            let path = workspace_root.join(change.relative_path);
            to_download.entry(path).or_insert(change.hash);
        }
    }
    let mut errors: Vec<String> = Vec::new();
    for (path, latest_hash) in to_download {
        match sync_file(&workspace_spec.repository, &path, &latest_hash) {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }
    workspace_branch.head = repo_branch.head;
    if let Err(e) = save_current_branch(&workspace_root, &workspace_branch) {
        errors.push(e);
    }
    if !errors.is_empty() {
        let message = errors.join("\n");
        return Err(message);
    }
    Ok(())
}
