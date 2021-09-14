use crate::{sql::*, *};
use std::fs;
use std::path::Path;
use url::Url;

pub async fn init_workspace_command(
    specified_workspace_directory: &Path,
    repo_location: &str,
) -> Result<(), String> {
    trace_scope!();
    let workspace_directory = make_path_absolute(specified_workspace_directory);

    let lsc_dir = workspace_directory.join(".lsc");
    let db_path = lsc_dir.join("workspace.db3");
    let db_uri = format!("sqlite://{}", db_path.display());

    let repo_addr = if Path::new(repo_location).exists() {
        RepositoryAddr::Local(make_path_absolute(Path::new(repo_location)))
    } else {
        match Url::parse(repo_location) {
            Ok(_uri) => RepositoryAddr::Remote(String::from(repo_location)),
            Err(e) => {
                return Err(format!(
                    "invalid repository location {}: {}",
                    repo_location, e
                ));
            }
        }
    };

    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repo_addr,
        root: String::from(workspace_directory.to_str().unwrap()),
        owner: whoami::username(),
    };

    let connection = connect_to_server(&spec).await?;

    if let Err(e) = fs::create_dir_all(&lsc_dir) {
        return Err(format!("Error creating .lsc directory: {}", e));
    }
    create_database(&db_uri).await?;

    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_directory).await?;
    create_workspace_branch_table(workspace_connection.sql()).await?;
    init_local_changes_database(&mut workspace_connection).await?;
    init_resolve_pending_database(&mut workspace_connection).await?;
    init_branch_merge_pending_database(&mut workspace_connection).await?;

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
    insert_current_branch(
        workspace_connection.sql(),
        &main_branch.name,
        &main_branch.head,
    )
    .await?;
    let commit = query.read_commit(&main_branch.head).await?;
    download_tree(&connection, &workspace_directory, &commit.root_hash).await?;
    Ok(())
}
