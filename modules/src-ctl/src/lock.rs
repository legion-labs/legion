use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Lock {
    pub relative_path: String, //needs to have a stable representation across platforms because it seeds the hash
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

fn hash_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:X}", hasher.finalize())
}

fn save_lock(repo: &Path, lock: &Lock) -> Result<(), String> {
    let path = repo.join(format!(
        "lock_domains/{}/{}.json",
        lock.lock_domain_id,
        hash_string(&lock.relative_path),
    ));
    if path.exists() {
        return Err(format!("Lock {} already exists", path.display()));
    }
    match serde_json::to_string(&lock) {
        Ok(contents) => {
            if let Err(e) = write_new_file(&path, contents.as_bytes()) {
                return Err(format!("Error writing lock to {}: {}", path.display(), e));
            }
        }
        Err(e) => {
            return Err(format!("Error formatting lock spec: {}", e));
        }
    }
    Ok(())
}

pub enum SearchResult<T, E> {
    Ok(T),
    Err(E),
    None,
}

fn read_lock(
    repo: &Path,
    lock_domain_id: &str,
    canonical_relative_path: &str,
) -> SearchResult<Lock, String> {
    let path = repo.join(format!(
        "lock_domains/{}/{}.json",
        lock_domain_id,
        hash_string(&canonical_relative_path)
    ));
    if !path.exists() {
        return SearchResult::None;
    }
    match read_text_file(&path) {
        Ok(contents) => {
            let parsed: serde_json::Result<Lock> = serde_json::from_str(&contents);
            match parsed {
                Ok(lock) => SearchResult::Ok(lock),
                Err(e) => SearchResult::Err(format!(
                    "Error parsing lock entry {}: {}",
                    path.display(),
                    e
                )),
            }
        }
        Err(e) => SearchResult::Err(format!("Error reading lock file {}: {}", path.display(), e)),
    }
}

fn clear_lock(
    repo: &Path,
    lock_domain_id: &str,
    canonical_relative_path: &str,
) -> Result<(), String> {
    let path = repo.join(format!(
        "lock_domains/{}/{}.json",
        lock_domain_id,
        hash_string(&canonical_relative_path)
    ));
    if !path.exists() {
        return Err(format!(
            "Error clearing lock {}, file {} not found",
            &canonical_relative_path,
            path.display()
        ));
    }
    if let Err(e) = fs::remove_file(&path) {
        return Err(format!(
            "Error clearing lock {}: {}",
            &canonical_relative_path, e
        ));
    }
    Ok(())
}

fn read_locks(repo: &Path, lock_domain_id: &str) -> Result<Vec<Lock>, String> {
    let mut locks = Vec::new();
    let domain = repo.join(format!("lock_domains/{}", lock_domain_id));
    match domain.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<Lock> =
                            serde_json::from_str(&read_text_file(&entry.path())?);
                        match parsed {
                            Ok(lock) => {
                                locks.push(lock);
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Error parsing lock entry {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error reading lock domain entry: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading directory {}: {}",
                domain.display(),
                e
            ));
        }
    }
    Ok(locks)
}

fn make_canonical_relative_path(
    workspace_root: &Path,
    path_specified: &Path,
) -> Result<String, String> {
    let abs_path = make_path_absolute(path_specified);
    let relative_path = path_relative_to(&abs_path, &workspace_root)?;
    let canonical_relative_path = relative_path.to_str().unwrap().replace("\\", "/");
    Ok(canonical_relative_path)
}

pub fn lock_file_command(path_specified: &Path) -> Result<(), String> {
    let workspace_root = find_workspace_root(&path_specified)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    let lock = Lock {
        relative_path: make_canonical_relative_path(&workspace_root, path_specified)?,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace_spec.id.clone(),
        branch_name: repo_branch.name,
    };
    save_lock(&repo, &lock)
}

pub fn unlock_file_command(path_specified: &Path) -> Result<(), String> {
    let workspace_root = find_workspace_root(&path_specified)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    let relative_path = make_canonical_relative_path(&workspace_root, path_specified)?;
    clear_lock(&repo, &repo_branch.lock_domain_id, &relative_path)
}

pub fn list_locks_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    let locks = read_locks(&repo, &repo_branch.lock_domain_id)?;
    if locks.is_empty() {
        println!("no locks found in domain {}", &repo_branch.lock_domain_id);
    }
    for lock in locks {
        println!(
            "{} in branch {} owned by workspace {}",
            &lock.relative_path, &lock.branch_name, &lock.workspace_id
        );
    }
    Ok(())
}

pub fn assert_not_locked(workspace_root: &Path, path_specified: &Path) -> Result<(), String> {
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    let relative_path = make_canonical_relative_path(&workspace_root, path_specified)?;
    match read_lock(&repo, &repo_branch.lock_domain_id, &relative_path) {
        SearchResult::Ok(lock) => {
            if lock.branch_name == current_branch.name && lock.workspace_id == workspace_spec.id {
                Ok(()) //locked by this workspace on this branch - all good
            } else {
                Err(format!(
                    "File {} locked in branch {}, owned by workspace {}",
                    lock.relative_path, lock.branch_name, lock.workspace_id
                ))
            }
        }
        SearchResult::Err(e) => Err(format!(
            "Error validating that {} is lock-free: {}",
            path_specified.display(),
            e
        )),
        SearchResult::None => {
            println!("no lock found");
            Ok(())
        }
    }
}
