use futures::executor::block_on;
use sqlx::migrate::MigrateDatabase;
use sqlx::Executor;
use std::path::Path;

pub fn create_sqlite_database(db_path: &Path) -> Result<(), String> {
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
