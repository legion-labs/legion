use crate::*;
use std::fs;

pub fn init_remote_repository(
    blob_storage: &BlobStorageSpec,
    host: &str,
    username: &str,
    password: &str,
    database_name: &str,
) -> Result<RepositoryAddr, String> {
    let repo_uri = format!(
        "mysql://{}:{}@{}/{}",
        username, password, host, database_name
    );
    create_database(&repo_uri)?;
    match blob_storage{
        BlobStorageSpec::LocalDirectory(blob_dir) => {
            if let Err(e) = fs::create_dir_all(blob_dir) {
                return Err(format!(
                    "Error creating directory {}: {}",
                    blob_dir.display(),
                    e
                ));
            }
        }
        BlobStorageSpec::S3Uri(s3_spec) => {
            return Err(format!("s3: {}", s3_spec));
        }
    }

    let addr = RepositoryAddr {
        repo_uri,
        blob_store: blob_storage.clone(),
    };
    let mut repo_connection = RepositoryConnection::new(&addr)?;
    init_repo_database(&mut repo_connection)?;
    push_init_repo_data(&mut repo_connection)?;
    Ok(addr)
}
