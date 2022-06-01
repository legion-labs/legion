mod dal;
mod errors;
mod session;
mod space;

use std::time::Duration;

use log::LevelFilter;
use sqlx::ConnectOptions;

pub use errors::{Error, Result};

/// A Server implementation.
pub struct Server {
    dal: dal::MySqlDal,
}

impl Server {
    /// Builds a new server.
    ///
    /// This makes sure to run migrations.
    ///
    /// # Errors
    ///
    /// This function will return an error if the database connection cannot be
    /// established.
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut connect_options: sqlx::mysql::MySqlConnectOptions = database_url.parse()?;

        connect_options
            .log_slow_statements(LevelFilter::Warn, Duration::from_secs(1))
            .log_statements(LevelFilter::Debug);

        let sqlx_pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;

        let dal = dal::MySqlDal::new(sqlx_pool).await?;

        Ok(Self { dal })
    }
}
