use crate::{sql::*, *};
use http::Uri;
use std::fs;

pub async fn init_mysql_repo_db(
    blob_storage: &BlobStorageSpec,
    host: &str,
    username: &str,
    password: &str,
    database_name: &str,
) -> Result<String, String> {
    let repo_uri = format!(
        "mysql://{}:{}@{}/{}",
        username, password, host, database_name
    );
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
    let pool = alloc_sql_pool(&repo_uri)?;
    init_repo_database(&pool, &repo_uri, blob_storage)?;
    push_init_repo_data(&repo_uri).await?;
    Ok(repo_uri)
}

pub fn init_remote_repository_command(repo_uri: &str) -> Result<(), String> {
    let specified_uri = repo_uri.parse::<Uri>().unwrap();
    let mut path = String::from(specified_uri.path());
    let name = path.split_off(1); //remove leading /
    let request = ServerRequest::InitRepo(InitRepositoryRequest { name });
    let resp = execute_request(repo_uri, &request)?;
    println!("{}", resp);
    Ok(())
}
