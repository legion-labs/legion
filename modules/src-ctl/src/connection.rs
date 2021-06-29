use std::path::{Path, PathBuf};

pub struct Connection {
    repo_directory: PathBuf,
    metadata_connection: sqlite::Connection,
}

impl Connection {
    pub fn new(repo_directory: &Path) -> Result<Self, String> {
        let db_path = repo_directory.join("repo.db3");

        match sqlite::open(&db_path) {
            Err(e) => Err(format!(
                "Error opening database {}: {}",
                db_path.display(),
                e
            )),
            Ok(c) => Ok(Self {
                repo_directory: repo_directory.to_path_buf(),
                metadata_connection: c,
            }),
        }
    }

    pub fn sql_connection(&self) -> &sqlite::Connection {
        &self.metadata_connection
    }

    pub fn repository(&self) -> &Path {
        &self.repo_directory
    }
}

pub fn execute_sql(connection: &sqlite::Connection, sql: &str) -> Result<(), String> {
    if let Err(e) = connection.execute(sql) {
        return Err(format!("SQL error: {}", e));
    }
    Ok(())
}
