use crate::{sql::*, *};

use std::fs;
use std::path::Path;

async fn init_workspace_impl(
    specified_workspace_directory: &Path,
    repo_uri: &str,
) -> Result<(), String> {
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

    let connection = connect_to_server(&spec).await?;

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

    let query = connection.query();
    query.insert_workspace(&spec).await?;
    let main_branch = query.read_branch("main").await?;
    save_current_branch(&workspace_directory, &main_branch)?;
    let commit = query.read_commit(&main_branch.head).await?;
    download_tree(&connection, &workspace_directory, &commit.root_hash).await?;
    Ok(())
}

pub fn init_workspace_command(
    specified_workspace_directory: &Path,
    repo_uri: &str,
) -> Result<(), String> {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    tokio_runtime.block_on(init_workspace_impl(specified_workspace_directory, repo_uri))
}
