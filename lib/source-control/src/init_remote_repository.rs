use crate::{
    execute_request, init_repo_database, push_init_repo_data,
    sql::{create_database, SqlConnectionPool},
    sql_repository_query::Databases,
    trace_scope, validate_connection_to_bucket, BlobStorageSpec, InitRepositoryRequest,
    ServerRequest,
};
use std::{fs, sync::Arc};
use url::Url;

pub async fn init_mysql_repo_db(
    blob_storage: &BlobStorageSpec,
    db_uri: &str,
) -> Result<Arc<SqlConnectionPool>, String> {
    match blob_storage {
        BlobStorageSpec::LocalDirectory(blob_dir) => {
            if let Err(e) = fs::create_dir_all(blob_dir) {
                return Err(format!(
                    "Error creating directory {}: {}",
                    blob_dir.display(),
                    e
                ));
            }
        }
        BlobStorageSpec::S3Uri(s3uri) => {
            if let Err(e) = validate_connection_to_bucket(s3uri).await {
                return Err(format!("Error connecting to s3: {}", e));
            }
        }
    }
    create_database(db_uri).await?;
    let pool = Arc::new(SqlConnectionPool::new(db_uri).await?);
    let mut sql_connection = pool.acquire().await?;
    init_repo_database(&mut sql_connection, db_uri, blob_storage).await?;
    push_init_repo_data(pool.clone(), Databases::Mysql).await?;
    Ok(pool)
}

async fn init_http_repository_command(host: &str, port: u16, name: &str) -> Result<(), String> {
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

pub async fn init_remote_repository_command(repo: &str, blob: Option<&str>) -> Result<(), String> {
    trace_scope!();
    let repo_uri = Url::parse(repo).unwrap();
    let mut uri_path = String::from(repo_uri.path());
    let path = uri_path.split_off(1); //remove leading /
    match repo_uri.scheme() {
        "mysql" => match blob {
            Some(blob_uri) => {
                let blob_spec = BlobStorageSpec::from_uri(blob_uri)?;
                let _pool = init_mysql_repo_db(&blob_spec, repo).await?;
            }
            None => {
                return Err(String::from("blob storage uri not specified"));
            }
        },
        "lsc" => {
            let host = repo_uri.host().unwrap();
            let port = repo_uri.port().unwrap_or(80);
            return init_http_repository_command(&host.to_string(), port, &path).await;
        }
        unknown => {
            return Err(format!("Unknown repository scheme {}", unknown));
        }
    }
    Ok(())
}
