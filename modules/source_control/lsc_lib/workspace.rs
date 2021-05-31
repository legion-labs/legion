use crate::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: String, //a file lock will contain the workspace id
    pub repository: PathBuf,
    pub root: PathBuf,
    pub owner: String,
}

pub fn find_workspace_root(directory: &Path) -> Result<&Path, String> {
    if let Ok(_meta) = fs::metadata(directory.join(".lsc")) {
        return Ok(directory);
    }
    match directory.parent() {
        None => Err(String::from("workspace not found")),
        Some(parent) => find_workspace_root(parent),
    }
}

pub fn read_workspace_spec(workspace_root_dir: &Path) -> Result<Workspace, String> {
    let workspace_json_path = workspace_root_dir.join(".lsc/workspace.json");
    let parsed: serde_json::Result<Workspace> =
        serde_json::from_str(&read_text_file(&workspace_json_path)?);
    match parsed {
        Ok(spec) => Ok(spec),
        Err(e) => Err(format!(
            "Error reading workspace spec {:?}: {}",
            &workspace_json_path, e
        )),
    }
}

pub fn write_workspace_spec(path: &Path, spec: &Workspace) -> Result<(), String> {
    match serde_json::to_string(spec) {
        Ok(json_spec) => write_file(path, json_spec.as_bytes()),
        Err(e) => Err(format!("Error formatting workspace spec: {}", e)),
    }
}

pub struct TempPath{
    pub path: PathBuf,
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path){
            println!("Error deleting temp file {}: {}", self.path.display(), e);
        }
    }
}


pub fn download_temp_file(
    repo: &Path,
    workspace_root: &Path,
    blob_hash: &str,
) -> Result<TempPath, String> {
    let tmp_dir = workspace_root.join(".lsc/tmp");
    let temp_file_path = tmp_dir.join(blob_hash);
    download_blob(repo, &temp_file_path, &blob_hash)?;
    Ok(TempPath{path: temp_file_path})
}
