use crate::{sql::*, *};
use http::Uri; //todo: remove
use std::{fs, sync::Arc};

pub async fn init_mysql_repo_db(
    blob_storage: &BlobStorageSpec,
    db_server_uri: &str,
    database_name: &str,
) -> Result<Arc<SqlConnectionPool>, String> {
    let repo_uri = format!("{}/{}", db_server_uri, database_name);
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
    create_database(&repo_uri)?;
    let pool = Arc::new(SqlConnectionPool::new(&repo_uri).await?);
    let mut sql_connection = pool.acquire().await?;
    init_repo_database(&mut sql_connection, &repo_uri, blob_storage)?;
    push_init_repo_data(pool.clone()).await?;
    Ok(pool)
}

pub async fn init_remote_repository_command(repo_uri: &str) -> Result<(), String> {
    let specified_uri = repo_uri.parse::<Uri>().unwrap();
    let mut path = String::from(specified_uri.path());
    let name = path.split_off(1); //remove leading /
    let request = ServerRequest::InitRepo(InitRepositoryRequest { repo_name: name });
    let host = specified_uri.host().unwrap();
    let port = specified_uri.port_u16().unwrap_or(80);
    let http_url = format!("http://{}:{}/lsc", host, port);
    let resp = execute_request(&http_url, &request).await?;
    println!("{}", resp);
    Ok(())
}
