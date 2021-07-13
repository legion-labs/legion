use crate::*;
use futures::executor::block_on;
use sqlx::migrate::MigrateDatabase;
use std::fs;
use std::path::Path;

pub fn init_remote_repository(
    blob_dir: &Path,
    host: &str,
    username: &str,
    password: &str,
    database_name: &str,
) -> Result<(), String> {
    let repo_uri = format!(
        "mysql://{}:{}@{}/{}",
        username, password, host, database_name
    );
    if let Err(e) = block_on(sqlx::Any::create_database(&repo_uri)) {
        return Err(format!("Error creating database {}: {}", repo_uri, e));
    }
    if let Err(e) = fs::create_dir_all(blob_dir) {
        return Err(format!(
            "Error creating directory {}: {}",
            blob_dir.display(),
            e
        ));
    }

    let blob_uri = format!(
        "file://{}",
        make_path_absolute(blob_dir)
            .to_str()
            .unwrap()
            .replace("\\", "/")
    );

    println!("repository uri: {}", repo_uri);
    println!("blob store uri: {}", blob_uri);

    Ok(())
}
