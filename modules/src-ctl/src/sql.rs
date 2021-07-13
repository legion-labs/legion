use futures::executor::block_on;
use sqlx::migrate::MigrateDatabase;
use sqlx::Executor;

pub fn create_database(uri: &str) -> Result<(), String> {
    if let Err(e) = block_on(sqlx::Any::create_database(uri)) {
        return Err(format!("Error creating database {}: {}", uri, e));
    }
    Ok(())
}

pub fn database_exists(uri: &str) -> Result<bool, String> {
    match block_on(sqlx::Any::database_exists(uri)) {
        Ok(res) => Ok(res),
        Err(e) => Err(format!("Error searching for database {}: {}", uri, e)),
    }
}

pub fn drop_database(uri: &str) -> Result<(), String> {
    if let Err(e) = block_on(sqlx::Any::drop_database(uri)) {
        return Err(format!("Error dropping database {}: {}", uri, e));
    }
    Ok(())
}

pub fn execute_sql(connection: &mut sqlx::AnyConnection, sql: &str) -> Result<(), String> {
    if let Err(e) = block_on(connection.execute(sql)) {
        return Err(format!("SQL error: {}", e));
    }
    Ok(())
}
