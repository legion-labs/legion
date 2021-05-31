use crate::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalChange {
    pub id: String,
    pub relative_path: PathBuf,
    pub change_type: String, //edit, add, delete
}

pub fn find_local_changes(workspace_root: &Path) -> Result<Vec<LocalChange>, String> {
    let local_edits_dir = workspace_root.join(".lsc/local_edits");
    let mut res = Vec::new();
    match local_edits_dir.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<LocalChange> =
                            serde_json::from_str(&read_text_file(&entry.path())?);
                        match parsed {
                            Ok(edit) => {
                                res.push(edit);
                            }
                            Err(e) => {
                                return Err(format!("Error parsing {:?}: {}", entry.path(), e))
                            }
                        }
                    }
                    Err(e) => return Err(format!("Error reading local edit entry: {}", e)),
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading directory {:?}: {}",
                local_edits_dir, e
            ))
        }
    }
    Ok(res)
}

pub fn find_local_changes_command() -> Result<Vec<LocalChange>, String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    find_local_changes(&workspace_root)
}

pub fn clear_local_changes(workspace_root: &Path, local_changes: &[LocalChange]) {
    for change in local_changes {
        let change_path = workspace_root.join(format!(".lsc/local_edits/{}.json", &change.id));
        if let Err(e) = fs::remove_file(change_path) {
            println!(
                "Error clearing local change {}: {}",
                change.relative_path.display(),
                e
            );
        }
    }
}

pub fn track_new_file(file_to_add_specified: &Path) -> Result<(), String> {
    let file_to_add_buf = make_path_absolute(file_to_add_specified);
    let file_to_add = file_to_add_buf.as_path();
    match fs::metadata(file_to_add) {
        Ok(_file_metadata) => {
            match file_to_add.parent() {
                None => {
                    return Err(format!(
                        "Error finding parent workspace of {:?}",
                        file_to_add
                    ));
                }
                Some(parent) => {
                    let workspace_root = make_path_absolute(find_workspace_root(parent)?);
                    let local_edit_id = uuid::Uuid::new_v4().to_string();
                    let local_edit_obj_path =
                        workspace_root.join(format!(".lsc/local_edits/{}.json", local_edit_id));

                    //todo: lock the new file before recording the local change
                    let local_change = LocalChange {
                        id: local_edit_id.clone(),
                        relative_path: path_relative_to(file_to_add, workspace_root.as_path())?,
                        change_type: String::from("add"),
                    };

                    match serde_json::to_string(&local_change) {
                        Ok(json_spec) => {
                            write_file(local_edit_obj_path.as_path(), json_spec.as_bytes())?;
                        }
                        Err(e) => {
                            return Err(format!("Error formatting local change spec: {}", e));
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading file metadata {:?}: {}",
                file_to_add, e
            ))
        }
    }
    Ok(())
}
