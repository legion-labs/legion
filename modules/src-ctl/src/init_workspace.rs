use crate::{sql::*, *};

use std::fs;
use std::path::Path;

pub fn init_workspace(specified_workspace_directory: &Path, repo_uri: &str) -> Result<(), String> {
    let workspace_directory = make_path_absolute(specified_workspace_directory);

    let lsc_dir = workspace_directory.join(".lsc");
    let db_path = lsc_dir.join("workspace.db3");
    let db_uri = format!("sqlite://{}", db_path.display());

    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repo_uri: String::from(repo_uri),
        root: String::from(workspace_directory.to_str().unwrap()),
        owner: whoami::username(),
    };

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut connection = tokio_runtime.block_on(connect_to_server(&spec))?;

    if let Err(e) = fs::create_dir_all(&lsc_dir) {
        return Err(format!("Error creating .lsc directory: {}", e));
    }
    create_database(&db_uri)?;

    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_directory)?;
    init_local_changes_database(&mut workspace_connection)?;
    init_resolve_pending_database(&mut workspace_connection)?;
    init_branch_merge_pending_database(&mut workspace_connection)?;

    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/tmp")) {
        return Err(format!("Error creating .lsc/tmp directory: {}", e));
    }

    if let Err(e) = fs::create_dir_all(workspace_directory.join(".lsc/blob_cache")) {
        return Err(format!("Error creating .lsc/blob_cache directory: {}", e));
    }

    write_workspace_spec(
        workspace_directory.join(".lsc/workspace.json").as_path(),
        &spec,
    )?;

    save_new_workspace_to_repo(&mut connection, &spec)?;
    let main_branch = read_branch_from_repo(&mut connection, "main")?;
    save_current_branch(&workspace_directory, &main_branch)?;
    let commit = read_commit(&mut connection, &main_branch.head)?;
    tokio_runtime.block_on(download_tree(
        &mut connection,
        &workspace_directory,
        &commit.root_hash,
    ))?;
    Ok(())
}
