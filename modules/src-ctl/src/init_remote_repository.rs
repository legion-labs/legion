use crate::*;
use std::fs;
use std::path::Path;

pub fn init_remote_repository(
    blob_dir: &Path,
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
    if let Err(e) = fs::create_dir_all(blob_dir) {
        return Err(format!(
            "Error creating directory {}: {}",
            blob_dir.display(),
            e
        ));
    }

    let addr = RepositoryAddr {
        repo_uri,
        blob_dir: make_path_absolute(blob_dir),
    };
    let mut repo_connection = RepositoryConnection::new(&addr)?;
    init_repo_database(&mut repo_connection)?;
    push_init_repo_data(&mut repo_connection)?;
    Ok(addr)
}
