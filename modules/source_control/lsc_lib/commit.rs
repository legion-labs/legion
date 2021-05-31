use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct HashedChange {
    pub relative_path: PathBuf,
    pub hash: String,
    pub change_type: String, //edit, add, delete
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub id: String,
    pub changes: Vec<HashedChange>,
    pub root_hash: String,
    pub parents: Vec<String>,
}

impl Commit {
    pub fn new(changes: Vec<HashedChange>, root_hash: String, parents: Vec<String>) -> Commit {
        let id = uuid::Uuid::new_v4().to_string();
        Commit {
            id,
            changes,
            root_hash,
            parents,
        }
    }
}

pub fn save_commit(repo: &Path, commit: &Commit) -> Result<(), String> {
    let file_path = repo.join("commits").join(commit.id.to_owned() + ".json");
    match serde_json::to_string(&commit) {
        Ok(json) => {
            write_file(&file_path, json.as_bytes())?;
        }
        Err(e) => {
            return Err(format!("Error formatting commit {:?}: {}", commit, e));
        }
    }
    Ok(())
}

fn write_blob(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }

    match std::fs::File::create(file_path) {
        Err(e) => {
            return Err(format!("Error creating file {:?}: {}", file_path, e));
        }
        Ok(output_file) => match lz4::EncoderBuilder::new().level(10).build(output_file) {
            Err(e) => return Err(format!("Error building lz4 encoder: {}", e)),
            Ok(mut encoder) => {
                if let Err(e) = encoder.write(contents) {
                    return Err(format!("Error writing to lz4 encoder: {}", e));
                }
                if let (_w, Err(e)) = encoder.finish() {
                    return Err(format!("Error closing lz4 encoder: {}", e));
                }
                Ok(())
            }
        },
    }
}

fn upload_localy_edited_blobs(
    workspace_root: &Path,
    workspace_spec: &Workspace,
    local_changes: &[LocalChange],
) -> Result<Vec<HashedChange>, String> {
    let blob_dir = Path::new(&workspace_spec.repository).join("blobs");
    let mut res = Vec::<HashedChange>::new();
    for local_change in local_changes {
        let workspace_path = workspace_root.join(&local_change.relative_path);
        let local_file_contents = read_bin_file(&workspace_path)?;
        let hash = format!("{:X}", Sha256::digest(&local_file_contents));
        write_blob(&blob_dir.join(&hash), &local_file_contents)?;
        res.push(HashedChange {
            relative_path: local_change.relative_path.clone(),
            hash: hash.clone(),
            change_type: local_change.change_type.clone(),
        });
    }
    Ok(res)
}

fn make_local_files_read_only(
    workspace_root: &Path,
    changes: &[HashedChange],
) -> Result<(), String> {
    for change in changes {
        if change.change_type != "delete" {
            let full_path = workspace_root.join(&change.relative_path);
            match fs::metadata(&full_path) {
                Ok(meta) => {
                    let mut permissions = meta.permissions();
                    permissions.set_readonly(true);
                    if let Err(e) = fs::set_permissions(&full_path, permissions) {
                        return Err(format!(
                            "Error making file read only for {}: {}",
                            full_path.display(),
                            e
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error reading file metadata for {}: {}",
                        full_path.display(),
                        e
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn commit(_message: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let local_changes = find_local_changes(workspace_root)?;
    let hashed_changes =
        upload_localy_edited_blobs(workspace_root, &workspace_spec, &local_changes)?;

    let new_root_hash = update_tree_from_changes(
        Tree::empty(), //todo: take tree of current commit
        &hashed_changes,
        &workspace_spec.repository,
    )?;

    let commit = Commit {
        id: uuid::Uuid::new_v4().to_string(),
        changes: hashed_changes,
        root_hash: new_root_hash,
        parents: Vec::new(), //todo: use the current commit, which should be the head of the branch
    };
    save_commit(&workspace_spec.repository, &commit)?;

    //todo: update branch

    if let Err(e) = make_local_files_read_only(&workspace_root, &commit.changes) {
        println!("Error making local files read only: {}", e);
    }
    clear_local_changes(&workspace_root, &local_changes);
    Ok(())
}
