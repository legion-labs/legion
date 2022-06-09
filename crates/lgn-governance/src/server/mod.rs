mod dal;
mod errors;
mod permission;
mod role;
mod session;
mod space;
mod user;
mod workspace;

use std::time::Duration;

use http::request;
use log::LevelFilter;
use sqlx::ConnectOptions;

pub use errors::{Error, Result};

use crate::types::UserInfo;

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

    fn get_caller_user_info_from_parts(parts: &request::Parts) -> Result<UserInfo> {
        parts
            .extensions
            .get::<lgn_auth::UserInfo>()
            .cloned()
            .ok_or(Error::MissingAuthenticationInfo)
            .and_then(|user_info| user_info.try_into().map_err(Into::into))
    }
}
