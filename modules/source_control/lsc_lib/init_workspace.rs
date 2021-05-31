use crate::*;

use std::fs;
use std::path::Path;

pub fn init_workspace(
    workspace_directory: &Path,
    repository_directory: &Path,
) -> Result<(), String> {
    if fs::metadata(workspace_directory).is_ok() {
        return Err(format!("{:?} already exists", workspace_directory));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc")) {
        return Err(format!("Error creating .lsc directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/local_edits")) {
        return Err(format!("Error creating .lsc/local_edits directory: {}", e));
    }
    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repository: path_to_string(repository_directory),
        owner: whoami::username(),
    };
    //todo: record the workspace in the central database
    match serde_json::to_string(&spec) {
        Ok(json_spec) => {
            write_file(
                workspace_directory.join(".lsc/workspace.json").as_path(),
                json_spec.as_bytes(),
            )?;
        }
        Err(e) => {
            return Err(format!("Error formatting workspace spec: {}", e));
        }
    }
    Ok(())
}
