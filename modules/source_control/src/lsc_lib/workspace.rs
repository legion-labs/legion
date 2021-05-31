use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: String, //a file lock will contain the workspace id
    pub repository: String,
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
