use futures::executor::block_on;
use sqlx::Connection;
use std::path::Path;

pub struct RepositoryConnection {
    repo_directory: String, //to store blobs, will be replaced by a generic blob storage interface
    sql_connection: sqlx::AnyConnection,
}

impl RepositoryConnection {
    pub fn new(repo_directory: &str) -> Result<Self, String> {
        let db_path = Path::new(repo_directory).join("repo.db3");
        let url = format!("sqlite://{}", db_path.display());
        match block_on(sqlx::AnyConnection::connect(&url)) {
            Err(e) => Err(format!("Error opening database {}: {}", url, e)),
            Ok(c) => Ok(Self {
                repo_directory: String::from(repo_directory),
                sql_connection: c,
            }),
        }
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn repository(&self) -> &Path {
        Path::new(&self.repo_directory)
    }
}
