use crate::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub head: String, //commit id
}

impl Branch {
    pub fn new(name: String, head: String) -> Branch {
        Branch { name, head }
    }
}

pub fn save_branch(repo: &Path, branch: &Branch) -> Result<(), String> {
    let file_path = repo.join("branches").join(branch.name.to_owned() + ".json");
    match serde_json::to_string(branch) {
        Ok(json) => {
            write_file(&file_path, json.as_bytes())?;
        }
        Err(e) => {
            return Err(format!("Error formatting branch {:?}: {}", branch, e));
        }
    }
    Ok(())
}
