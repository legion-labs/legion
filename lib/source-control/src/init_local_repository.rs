use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::sql_repository_query::{Databases, SqlRepositoryQuery};
use crate::{
    create_branches_table, init_commit_database, init_config_database, init_forest_database,
    init_lock_database, init_workspace_database, insert_config, make_path_absolute,
    sql::{create_database, SqlConnectionPool},
    trace_scope, BlobStorageSpec, Branch, Commit, RepositoryQuery, Tree,
};

pub async fn init_repo_database(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage: &BlobStorageSpec,
) -> Result<()> {
    init_config_database(sql_connection).await?;
    init_commit_database(sql_connection).await?;
    init_forest_database(sql_connection).await?;
    create_branches_table(sql_connection).await?;
    init_workspace_database(sql_connection).await?;
    init_lock_database(sql_connection).await?;
    insert_config(sql_connection, self_uri, blob_storage).await?;

    Ok(())
}

pub async fn push_init_repo_data(
    pool: Arc<SqlConnectionPool>,
    database_kind: Databases,
) -> Result<()> {
    let query = SqlRepositoryQuery::new(pool, database_kind);
    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    query.save_tree(&root_tree, &root_hash).await?;

    let id = uuid::Uuid::new_v4().to_string();
    let initial_commit = Commit::new(
        id,
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    query.insert_commit(&initial_commit).await?;

    let main_branch = Branch::new(
        String::from("main"),
        initial_commit.id,
        String::new(),
        lock_domain_id,
    );
    query.insert_branch(&main_branch).await?;
    Ok(())
}

pub async fn init_local_repository_command(directory: &Path) -> Result<()> {
    trace_scope!();

    if fs::metadata(directory).is_ok() {
        anyhow::bail!("{} already exists", directory.display());
    }

    fs::create_dir_all(&directory).context("error creating repository directory")?;

    let db_path = make_path_absolute(&directory.join("repo.db3"));
    let repo_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    create_database(&repo_uri).await?;

    let blob_dir = make_path_absolute(&directory.join("blobs"));

    fs::create_dir_all(&blob_dir).context("error creating blob directory")?;

    let pool = Arc::new(SqlConnectionPool::new(&repo_uri).await?);

    let mut sql_connection = pool.acquire().await?;
    init_repo_database(
        &mut sql_connection,
        &repo_uri,
        &BlobStorageSpec::LocalDirectory(blob_dir),
    )
    .await?;

    push_init_repo_data(pool, Databases::Sqlite).await?;

    Ok(())
}
