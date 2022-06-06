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
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
