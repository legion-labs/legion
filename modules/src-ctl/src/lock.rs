use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Lock {
    pub relative_path: String, //needs to have a stable representation across platforms because it seeds the hash
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

impl Lock {
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.relative_path.as_bytes());
        format!("{:X}", hasher.finalize())
    }
}

fn save_lock(repo: &Path, lock: &Lock) -> Result<(), String> {
    let path = repo.join(format!(
        "lock_domains/{}/{}.json",
        lock.lock_domain_id,
        lock.hash()
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

pub fn lock_file_command(path_specified: &Path) -> Result<(), String> {
    let workspace_root = find_workspace_root(&path_specified)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let repo_branch = read_branch_from_repo(repo, &current_branch.name)?;
    let abs_path = make_path_absolute(path_specified);
    let relative_path = path_relative_to(&abs_path, &workspace_root)?;
    let canonical_relative_path = relative_path.to_str().unwrap().replace("\\", "/");
    let lock = Lock {
        relative_path: canonical_relative_path,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace_spec.id.clone(),
        branch_name: repo_branch.name.clone(),
    };
    save_lock(&repo, &lock)
}
