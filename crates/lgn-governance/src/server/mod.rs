mod aws_cognito_dal;
mod errors;
mod mysql_dal;
mod permission;
mod permissions_cache;
mod role;
mod session;
mod space;
mod user;
mod workspace;

use std::{borrow::Cow, sync::Arc, time::Duration};

use http::request;
use log::LevelFilter;
use sqlx::ConnectOptions;

pub use errors::{Error, Result};

use crate::types::UserId;
pub use permissions_cache::{PermissionsCache, PermissionsProvider};

/// A Server implementation.
pub struct Server {
    init_key: String,
    mysql_dal: Arc<mysql_dal::MySqlDal>,
    aws_cognito_dal: aws_cognito_dal::AwsCognitoDal,
    permissions_cache: PermissionsCache<Arc<mysql_dal::MySqlDal>>,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub init_key: String,
    pub mysql: ServerMySqlOptions,
    pub aws_cognito: ServerAwsCognitoOptions,
}

#[derive(Debug, Clone)]
pub struct ServerMySqlOptions {
    pub database_url: String,
}

#[derive(Debug, Clone)]
pub struct ServerAwsCognitoOptions {
    pub region: Option<Cow<'static, str>>,
    pub user_pool_id: String,
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
    pub async fn new(options: ServerOptions) -> Result<Self> {
        if options.init_key.is_empty() {
            return Err(Error::Configuration("No init key was set".to_string()));
        }

        let mut connect_options: sqlx::mysql::MySqlConnectOptions =
            options.mysql.database_url.parse()?;

        connect_options
            .log_slow_statements(LevelFilter::Warn, Duration::from_secs(1))
            .log_statements(LevelFilter::Debug);

        let sqlx_pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;

        let mysql_dal = Arc::new(mysql_dal::MySqlDal::new(sqlx_pool).await?);
        let permissions_cache = PermissionsCache::new(Arc::clone(&mysql_dal));
        let aws_cognito_dal = aws_cognito_dal::AwsCognitoDal::new(
            options.aws_cognito.region,
            options.aws_cognito.user_pool_id,
        )
        .await?;

        Ok(Self {
            init_key: options.init_key,
            mysql_dal,
            aws_cognito_dal,
            permissions_cache,
        })
    }

    fn get_caller_user_id_from_parts(parts: &request::Parts) -> Result<UserId> {
        parts
            .extensions
            .get::<lgn_auth::UserInfo>()
            .cloned()
            .ok_or(Error::Unauthorized)
            .and_then(|user_info| {
                user_info.username.ok_or_else(|| {
                    Error::Unexpected("authorization token contains no `username`".to_string())
                })
            })
            .and_then(|s| s.parse().map_err(Into::into))
    }
}
