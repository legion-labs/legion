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
