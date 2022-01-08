use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use lgn_telemetry::{info, trace_function};
use reqwest::Url;

use crate::http_repository_query::HttpRepositoryQuery;
use crate::sql_repository_query::{Databases, SqlRepositoryQuery};
use crate::{
    check_directory_does_not_exist_or_is_empty, create_branches_table, init_commit_database,
    init_config_database, init_forest_database, init_lock_database, init_workspace_database,
    insert_config, make_path_absolute,
    sql::{create_database, SqlConnectionPool},
    BlobStorageUrl, Branch, Commit, RepositoryQuery, Tree,
};
use crate::{RepositoryConnection, RepositoryUrl};

/// Create a repository at the specified URL.
///
/// If no blob storage url is specified and the repository URL has a default
/// associated blob storage URL, it will be used.
pub async fn create_repository(
    repo_url: &RepositoryUrl,
    blob_storage_url: &Option<BlobStorageUrl>,
) -> Result<RepositoryConnection> {
    info!("Creating repository at {}", repo_url);

    if let Some(blob_storage_url) = blob_storage_url {
        info!("Using blob storage at {}", blob_storage_url);
    } else {
        info!("No blob storage specified, using default blob storage for the repository type");
    }

    match repo_url {
        RepositoryUrl::Local(directory) => {
            create_local_repository(directory, blob_storage_url).await
        }
        RepositoryUrl::MySQL(url) => create_mysql_repository(url, blob_storage_url).await,
        RepositoryUrl::Lsc(url) => create_lsc_repository(url).await,
    }
}

#[trace_function]
async fn create_local_repository(
    directory: impl AsRef<Path>,
    blob_storage_url: &Option<BlobStorageUrl>,
) -> Result<RepositoryConnection> {
    let directory = directory.as_ref();

    check_directory_does_not_exist_or_is_empty(directory)?;

    info!("Creating repository root at {}", directory.display());

    fs::create_dir_all(directory).context("could not create repository directory")?;

    let directory = make_path_absolute(directory);
    let db_path = directory.join("repo.db3");

    info!("SQLite database lives at {}", db_path.display());

    let repo_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));

    info!("Creating SQLite database at {}", &repo_uri);

    create_database(&repo_uri).await?;

    let default_blob_storage_url = &BlobStorageUrl::Local(directory.join("blobs"));
    let blob_storage_url = blob_storage_url
        .as_ref()
        .unwrap_or(default_blob_storage_url);

    let pool = Arc::new(SqlConnectionPool::new(repo_uri.clone()).await?);
    let mut sql_connection = pool.acquire().await?;

    init_repo_database(&mut sql_connection, repo_uri.as_str(), blob_storage_url).await?;
    push_init_repo_data(pool.clone(), Databases::Sqlite).await?;

    let repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Sqlite));
    let blob_storage = blob_storage_url.clone().into_blob_storage().await?;

    Ok(RepositoryConnection {
        repo_query,
        blob_storage,
    })
}

#[trace_function]
async fn create_mysql_repository(
    url: &Url,
    blob_storage_url: &Option<BlobStorageUrl>,
) -> Result<RepositoryConnection> {
    let blob_storage_url = blob_storage_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "refusing to create a MySQL repository as no blob storage URL was specified"
        )
    })?;

    create_database(url.as_str()).await?;

    let pool = Arc::new(SqlConnectionPool::new(url.to_string()).await?);
    let mut sql_connection = pool.acquire().await?;

    init_repo_database(&mut sql_connection, url.as_str(), blob_storage_url).await?;
    push_init_repo_data(pool.clone(), Databases::Mysql).await?;

    let repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Mysql));
    let blob_storage = blob_storage_url.clone().into_blob_storage().await?;

    Ok(RepositoryConnection {
        repo_query,
        blob_storage,
    })
}

#[trace_function]
async fn create_lsc_repository(url: &Url) -> Result<RepositoryConnection> {
    let repo_name = url.path().trim_start_matches('/');
    let mut url = url.clone();
    url.set_path("");

    let repo_query = Box::new(HttpRepositoryQuery::new(url, repo_name.to_string()));
    let blob_storage = repo_query
        .create_repository(repo_name)
        .await?
        .into_blob_storage()
        .await?;

    Ok(RepositoryConnection {
        repo_query,
        blob_storage,
    })
}

async fn init_repo_database(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage_url: &BlobStorageUrl,
) -> Result<()> {
    init_config_database(sql_connection).await?;
    init_commit_database(sql_connection).await?;
    init_forest_database(sql_connection).await?;
    create_branches_table(sql_connection).await?;
    init_workspace_database(sql_connection).await?;
    init_lock_database(sql_connection).await?;
    insert_config(sql_connection, self_uri, blob_storage_url).await?;

    Ok(())
}

async fn push_init_repo_data(pool: Arc<SqlConnectionPool>, database_kind: Databases) -> Result<()> {
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
