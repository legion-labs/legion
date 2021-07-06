use crate::*;

use std::fs;
use std::path::Path;

pub fn init_workspace(
    specified_workspace_directory: &Path,
    specified_repository_directory: &Path,
) -> Result<(), String> {
    let workspace_directory = make_path_absolute(specified_workspace_directory);
    let repository_directory = make_path_absolute(specified_repository_directory);
    let mut connection = RepositoryConnection::new(&repository_directory)?;
    if fs::metadata(&workspace_directory).is_ok() {
        return Err(format!("{} already exists", workspace_directory.display()));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc")) {
        return Err(format!("Error creating .lsc directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/local_edits")) {
        return Err(format!("Error creating .lsc/local_edits directory: {}", e));
    }
    //todo rename resolve_pending
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/resolve_pending")) {
        return Err(format!(
            "Error creating .lsc/resolve_pending directory: {}",
            e
        ));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/tmp")) {
        return Err(format!("Error creating .lsc/tmp directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/branch_merge_pending")) {
        return Err(format!(
            "Error creating .lsc/branch_merge_pending directory: {}",
            e
        ));
    }
    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repository: repository_directory.clone(),
        root: workspace_directory.clone(),
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
    let main_branch = read_branch_from_repo(&mut connection, "main")?;
    save_current_branch(&workspace_directory, &main_branch)?;
    let commit = read_commit(&mut connection, &main_branch.head)?;
    download_tree(&mut connection, &workspace_directory, &commit.root_hash)?;
    Ok(())
}
