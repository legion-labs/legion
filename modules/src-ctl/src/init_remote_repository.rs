use crate::*;
use std::fs;

pub fn init_remote_repository(
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
            if let Err(e) = validate_connection_to_bucket(s3uri) {
                return Err(format!("Error connecting to s3: {}", e));
            }
        }
    }
    create_database(&repo_uri)?;
    let mut sql_connection = connect(&repo_uri)?;
    init_repo_database(&mut sql_connection)?;
    push_init_repo_data(&mut sql_connection, &repo_uri, blob_storage)?;
    Ok(repo_uri)
}
