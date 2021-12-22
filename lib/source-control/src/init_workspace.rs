use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use url::Url;

use crate::{
    connect_to_server, create_workspace_branch_table, download_tree,
    init_branch_merge_pending_database, init_local_changes_database, init_resolve_pending_database,
    insert_current_branch, make_path_absolute, sql::create_database, trace_scope,
    write_workspace_spec, LocalWorkspaceConnection, RepositoryAddr, Workspace,
};

pub async fn init_workspace_command(
    specified_workspace_directory: &Path,
    repo_location: &str,
) -> Result<()> {
    trace_scope!();
    let workspace_directory = make_path_absolute(specified_workspace_directory);

    let lsc_dir = workspace_directory.join(".lsc");
    let db_path = lsc_dir.join("workspace.db3");
    let db_uri = format!("sqlite://{}", db_path.display());

    let repo_addr = if Path::new(repo_location).exists() {
        RepositoryAddr::Local(make_path_absolute(Path::new(repo_location)))
    } else {
        Url::parse(repo_location).context(format!("invalid repo location: {}", repo_location))?;

        RepositoryAddr::Remote(String::from(repo_location))
    };

    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repo_addr,
        root: String::from(workspace_directory.to_str().unwrap()),
        owner: whoami::username(),
    };

    let connection = connect_to_server(&spec).await?;

    fs::create_dir_all(&lsc_dir).context("failed to create `.lsc` directory")?;
    create_database(&db_uri).await?;

    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_directory).await?;
    create_workspace_branch_table(workspace_connection.sql()).await?;
    init_local_changes_database(&mut workspace_connection).await?;
    init_resolve_pending_database(&mut workspace_connection).await?;
    init_branch_merge_pending_database(&mut workspace_connection).await?;

    fs::create_dir_all(workspace_directory.join(".lsc/tmp"))
        .context("failed to create `.lsc/tmp` directory")?;
    fs::create_dir_all(workspace_directory.join(".lsc/blob_cache"))
        .context("failed to create `.lsc/blob_cache` directory")?;

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
