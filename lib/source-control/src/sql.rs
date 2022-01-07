use anyhow::{Context, Result};
use sqlx::migrate::MigrateDatabase;
use sqlx::Executor;

pub async fn create_database(uri: &str) -> Result<()> {
    sqlx::Any::create_database(uri)
        .await
        .context("error creating database")
}

pub async fn database_exists(uri: &str) -> Result<bool> {
    sqlx::Any::database_exists(uri)
        .await
        .context(format!("error checking database `{}` exists", uri))
}

pub async fn drop_database(uri: &str) -> Result<()> {
    sqlx::Any::drop_database(uri)
        .await
        .context(format!("error dropping database `{}`", uri))
}

pub async fn execute_sql(connection: &mut sqlx::AnyConnection, sql: &str) -> Result<()> {
    connection.execute(sql).await.context("SQL error")?;

    Ok(())
}

#[derive(Debug)]
pub struct SqlConnectionPool {
    pub pool: sqlx::AnyPool,
    pub database_uri: String,
}

impl SqlConnectionPool {
    pub async fn new(database_uri: String) -> Result<Self> {
        Ok(Self {
            pool: alloc_sql_pool(&database_uri).await?,
            database_uri,
        })
    }

    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.pool
            .acquire()
            .await
            .context("error acquiring connection")
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Any>> {
        self.pool
            .begin()
            .await
            .context("error beginning transaction")
    }
}

pub async fn alloc_sql_pool(url: &str) -> Result<sqlx::AnyPool> {
    sqlx::any::AnyPoolOptions::new()
        .connect(url)
        .await
        .context("error allocating database pool")
}
