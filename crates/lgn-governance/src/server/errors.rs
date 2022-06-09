use axum::response::IntoResponse;
use http::StatusCode;
use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migrate: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("types: {0}")]
    Types(#[from] crate::types::Error),
    #[error("the authentication info is missing")]
    MissingAuthenticationInfo,
}

impl From<Error> for lgn_online::server::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(err) => Self::Internal(err.to_string()),
            Error::SqlxMigrate(err) => Self::Internal(err.to_string()),
            Error::Types(err) => Self::Internal(err.to_string()),
            Error::MissingAuthenticationInfo => Self::Custom(
                (StatusCode::UNAUTHORIZED, "Missing authentication info").into_response(),
            ),
        }
    }
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
