use crate::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct MergePending {
    pub id: String,
    pub relative_path: PathBuf,
    pub base_version: String,
    pub theirs_version: String,
}

impl MergePending {
    pub fn new(
        relative_path: PathBuf,
        base_version: String,
        theirs_version: String,
    ) -> MergePending {
        let id = uuid::Uuid::new_v4().to_string();
        MergePending {
            id,
            relative_path,
            base_version,
            theirs_version,
        }
    }
}

pub fn save_merge_pending(
    workspace_root: &Path,
    merge_pending: &MergePending,
) -> Result<(), String> {
    let file_path = workspace_root.join(format!(".lsc/merge_pending/{}.json", &merge_pending.id));
    match serde_json::to_string(&merge_pending) {
        Ok(json) => {
            write_file(&file_path, json.as_bytes())?;
        }
        Err(e) => {
            return Err(format!("Error formatting merge pending: {}", e));
        }
    }
    Ok(())
}

fn read_merges_pending(workspace_root: &Path) -> Result<Vec<MergePending>, String>{
    let merges_pending_dir = workspace_root.join(".lsc/merge_pending");
    let mut res = Vec::new();
    match merges_pending_dir.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<MergePending> =
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
                    Err(e) => return Err(format!("Error reading merge pending entry: {}", e)),
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading directory {:?}: {}",
                merges_pending_dir, e
            ))
        }
    }
    Ok(res)
}

pub fn find_merges_pending_command() -> Result<Vec<MergePending>, String>{
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    read_merges_pending(&workspace_root)
}
