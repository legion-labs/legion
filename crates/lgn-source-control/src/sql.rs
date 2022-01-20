use sqlx::migrate::MigrateDatabase;
use sqlx::Executor;

use crate::{MapOtherError, Result};

pub async fn create_database(uri: &str) -> Result<()> {
    sqlx::Any::create_database(uri)
        .await
        .map_other_err("failed to create database")
}

pub async fn database_exists(uri: &str) -> Result<bool> {
    sqlx::Any::database_exists(uri)
        .await
        .map_other_err("failed to check if database exists")
}

pub async fn drop_database(uri: &str) -> Result<()> {
    sqlx::Any::drop_database(uri)
        .await
        .map_other_err("failed to drop database")
}

pub async fn execute_sql(connection: &mut sqlx::AnyConnection, sql: &str) -> Result<()> {
    connection
        .execute(sql)
        .await
        .map_other_err("failed to execute SQL query")
        .map(|_| ())
}

#[derive(Debug)]
pub struct SqlConnectionPool {
    pub pool: sqlx::AnyPool,
}

impl SqlConnectionPool {
    pub async fn new(database_uri: &str) -> Result<Self> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .after_connect(|c| {
                Box::pin(async move {
                    println!("after_connect");
                    Ok(())
                })
            })
            .after_release(|c| {
                println!("after_release");
                true
            })
            .before_acquire(|c| {
                Box::pin(async move {
                    println!("before acquire");
                    Ok(true)
                })
            })
            .connect(database_uri)
            .await
            .map_other_err("failed to allocate SQL connection pool")?;

        Ok(Self { pool })
    }

    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.pool
            .acquire()
            .await
            .map_other_err("failed to acquire connection from pool")
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Any>> {
        self.pool
            .begin()
            .await
            .map_other_err("failed to start transaction")
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
