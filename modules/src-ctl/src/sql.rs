use futures::executor::block_on;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, Executor};

pub fn create_database(uri: &str) -> Result<(), String> {
    if let Err(e) = block_on(sqlx::Any::create_database(uri)) {
        //don't print uri, could contain user/passwd of database
        return Err(format!("Error creating database: {}", e));
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

pub fn connect(database_uri: &str) -> Result<sqlx::AnyConnection, String> {
    match block_on(sqlx::AnyConnection::connect(database_uri)) {
        Ok(connection) => Ok(connection),
        Err(e) => Err(format!("Error connecting to database: {}", e)),
    }
}

#[derive(Debug)]
pub struct SqlConnectionPool {
    pub pool: sqlx::AnyPool,
}

pub fn alloc_sql_pool(database_uri: &str) -> Result<sqlx::AnyPool, String> {
    match block_on(
        sqlx::any::AnyPoolOptions::new()
            .max_connections(5)
            .connect(database_uri),
    ) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(format!("Error allocating database pool: {}", e)),
    }
}

pub fn make_sql_connection_pool(database_uri: &str) -> Result<SqlConnectionPool, String> {
    Ok(SqlConnectionPool {
        pool: alloc_sql_pool(database_uri)?,
    })
}
