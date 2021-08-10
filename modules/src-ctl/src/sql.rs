use sqlx::migrate::MigrateDatabase;
use sqlx::Executor;

pub async fn create_database(uri: &str) -> Result<(), String> {
    if let Err(e) = sqlx::Any::create_database(uri).await {
        //don't print uri, could contain user/passwd of database
        return Err(format!("Error creating database: {}", e));
    }
    Ok(())
}

pub async fn database_exists(uri: &str) -> Result<bool, String> {
    match sqlx::Any::database_exists(uri).await {
        Ok(res) => Ok(res),
        Err(e) => Err(format!("Error searching for database {}: {}", uri, e)),
    }
}

pub async fn drop_database(uri: &str) -> Result<(), String> {
    if let Err(e) = sqlx::Any::drop_database(uri).await {
        return Err(format!("Error dropping database {}: {}", uri, e));
    }
    Ok(())
}

pub async fn execute_sql(connection: &mut sqlx::AnyConnection, sql: &str) -> Result<(), String> {
    if let Err(e) = connection.execute(sql).await {
        return Err(format!("SQL error: {}", e));
    }
    Ok(())
}

#[derive(Debug)]
pub struct SqlConnectionPool {
    pub pool: sqlx::AnyPool,
    pub database_uri: String,
}

impl SqlConnectionPool {
    pub async fn new(database_uri: &str) -> Result<Self, String> {
        Ok(Self {
            pool: alloc_sql_pool(database_uri).await?,
            database_uri: String::from(database_uri),
        })
    }

    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>, String> {
        match self.pool.acquire().await {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("Error acquiring sql connection: {}", e)),
        }
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Any>, String> {
        match self.pool.begin().await {
            Ok(t) => Ok(t),
            Err(e) => Err(format!("Error beginning sql transaction: {}", e)),
        }
    }
}

pub async fn alloc_sql_pool(db_server_uri: &str) -> Result<sqlx::AnyPool, String> {
    match sqlx::any::AnyPoolOptions::new()
        .connect(db_server_uri)
        .await
    {
        Ok(pool) => Ok(pool),
        Err(e) => Err(format!("Error allocating database pool: {}", e)),
    }
}
