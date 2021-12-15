use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Url;

use crate::sql_repository_query::{Databases, SqlRepositoryQuery};
use crate::{
    check_directory_does_not_exist_or_is_empty, create_branches_table, init_commit_database,
    init_config_database, init_forest_database, init_lock_database, init_workspace_database,
    insert_config, make_path_absolute,
    sql::{create_database, SqlConnectionPool},
    trace_scope, BlobStorageUrl, Branch, Commit, RepositoryQuery, Tree,
};
use crate::{
    execute_request, validate_connection_to_bucket, InitRepositoryRequest, RepositoryUrl,
    ServerRequest,
};

/// Create a repository at the specified URL.
///
/// If no blob storage url is specified and the repository URL has a default
/// associated blob storage URL, it will be used.
pub async fn create_repository(
    repo_url: &RepositoryUrl,
    blob_storage_url: &Option<BlobStorageUrl>,
) -> Result<()> {
    match repo_url {
        RepositoryUrl::Local(directory) => {
            create_local_repository(directory, blob_storage_url).await
        }
        _ => {
            anyhow::bail!("unsupported repository URL: {}", repo_url)
        }
    }
}

async fn create_local_repository(
    directory: impl AsRef<Path>,
    blob_storage_url: &Option<BlobStorageUrl>,
) -> Result<()> {
    trace_scope!();

    let directory = directory.as_ref();

    check_directory_does_not_exist_or_is_empty(directory)?;
    fs::create_dir_all(directory).context("could not create repository directory")?;

    let directory = make_path_absolute(directory);
    let db_path = directory.join("repo.db3");
    let repo_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    create_database(&repo_uri).await?;

    let default_blob_storage_url = &BlobStorageUrl::Local(directory.join("blobs"));
    let blob_storage_url = blob_storage_url
        .as_ref()
        .unwrap_or(default_blob_storage_url);

    // TODO: Implement a blob storage backend for the local filesystem.

    if let BlobStorageUrl::Local(blobs_dir) = &blob_storage_url {
        fs::create_dir_all(blobs_dir).context("could not create blobs directory")?;
    }

    let pool = Arc::new(SqlConnectionPool::new(&repo_uri).await?);

    let mut sql_connection = pool.acquire().await?;

    init_repo_database(&mut sql_connection, &repo_uri, blob_storage_url).await?;

    push_init_repo_data(pool, Databases::Sqlite).await?;

    Ok(())
}

async fn init_repo_database(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage: &BlobStorageUrl,
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

async fn init_mysql_repo_db(
    blob_storage: &BlobStorageUrl,
    db_uri: &str,
) -> Result<Arc<SqlConnectionPool>> {
    match blob_storage {
        BlobStorageUrl::Local(blob_dir) => {
            fs::create_dir_all(blob_dir).context(format!(
                "failed to create blobs dir: {}",
                blob_dir.display()
            ))?;
        }
        BlobStorageUrl::AwsS3(s3uri) => {
            validate_connection_to_bucket(s3uri)
                .await
                .context(format!("failed to connect to AWS S3 bucket: {}", s3uri))?;
        }
    }

    create_database(db_uri).await?;

    let pool = Arc::new(SqlConnectionPool::new(db_uri).await?);
    let mut sql_connection = pool.acquire().await?;

    init_repo_database(&mut sql_connection, db_uri, blob_storage).await?;
    push_init_repo_data(pool.clone(), Databases::Mysql).await?;

    Ok(pool)
}

async fn init_http_repository_command(host: &str, port: u16, name: &str) -> Result<()> {
    trace_scope!();
    let request = ServerRequest::InitRepo(InitRepositoryRequest {
        repo_name: String::from(name),
    });
    let http_url = format!("http://{}:{}/lsc", host, port);
    let client = reqwest::Client::new();
    let resp = execute_request(&client, &http_url, &request).await?;
    println!("{}", resp);
    Ok(())
}

pub async fn create_remote_repository_command(repo: &str, blob: Option<&str>) -> Result<()> {
    trace_scope!();
    let repo_uri = Url::parse(repo).unwrap();
    let mut uri_path = String::from(repo_uri.path());
    let path = uri_path.split_off(1); //remove leading /
    match repo_uri.scheme() {
        "mysql" => {
            if let Some(blob_uri) = blob {
                let blob_spec = blob_uri.parse()?;
                let _pool = init_mysql_repo_db(&blob_spec, repo).await?;
            } else {
                anyhow::bail!("missing blob storage spec");
            }
        }
        "lsc" => {
            let host = repo_uri.host().unwrap();
            let port = repo_uri.port().unwrap_or(80);
            return init_http_repository_command(&host.to_string(), port, &path).await;
        }
        unknown => {
            anyhow::bail!("unknown repository scheme: {}", unknown);
        }
    }

    Ok(())
}
