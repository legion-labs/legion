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
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/merge_pending")) {
        return Err(format!("Error creating .lsc/merge_pending directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/tmp")) {
        return Err(format!("Error creating .lsc/tmp directory: {}", e));
    }
    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repository: repository_directory.to_path_buf(),
        root: workspace_directory.to_path_buf(),
        owner: whoami::username(),
    };
    write_workspace_spec(
        workspace_directory.join(".lsc/workspace.json").as_path(),
        &spec,
    )?;
    write_workspace_spec(
        repository_directory
            .join(format!("workspaces/{}.json", &spec.id))
            .as_path(),
        &spec,
    )?;
    let main_branch = read_branch(repository_directory.join("branches/main.json").as_path())?;
    save_current_branch(workspace_directory, &main_branch)?;
    let commit = read_commit(repository_directory, &main_branch.head)?;
    download_tree(repository_directory, workspace_directory, &commit.root_hash)?;
    Ok(())
}
