use futures::executor::block_on;
use sqlx::migrate::MigrateDatabase;
use sqlx::Connection;
use sqlx::Executor;
use std::path::{Path, PathBuf};

pub struct RepositoryConnection {
    repo_directory: PathBuf,
    // metadata_connection: sqlite::Connection,
    metadata_connection: sqlx::AnyConnection,
}

pub type Statement<'a> = <sqlx::Any as sqlx::database::HasStatement<'a>>::Statement;

impl RepositoryConnection {
    pub fn new(repo_directory: &Path) -> Result<Self, String> {
        let db_path = repo_directory.join("repo.db3");
        let url = format!("sqlite://{}", db_path.display());
        match block_on(sqlx::AnyConnection::connect(&url)) {
            Err(e) => Err(format!("Error opening database {}: {}", url, e)),
            Ok(c) => Ok(Self {
                repo_directory: repo_directory.to_path_buf(),
                metadata_connection: c,
            }),
        }
    }

    pub fn sql_connection(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.metadata_connection
    }

    pub fn repository(&self) -> &Path {
        &self.repo_directory
    }
}

pub fn create_sqlite_repo_database(repo_directory: &Path) -> Result<(), String> {
    let db_path = repo_directory.join("repo.db3");
    let url = format!("sqlite://{}", db_path.display());
    if let Err(e) = block_on(sqlx::Any::create_database(&url)) {
        return Err(format!("Error creating database {}: {}", url, e));
    }
    Ok(())
}

pub fn execute_sql(connection: &mut sqlx::AnyConnection, sql: &str) -> Result<(), String> {
    if let Err(e) = block_on(connection.execute(sql)) {
        return Err(format!("SQL error: {}", e));
    }
    Ok(())
}

pub fn prepare_statement<'q>(
    connection: &mut sqlx::AnyConnection,
    sql: &'q str,
) -> Result<Statement<'q>, String> {
    match block_on(connection.prepare(sql)) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("Error preparing statement {}: {}", sql, e)),
    }
}
