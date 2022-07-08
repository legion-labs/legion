use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("blob storage: {0}")]
    BlobStorage(#[from] lgn_blob_storage::Error),
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migrate: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("unexpected: {0}")]
    Unexpected(String),
}

impl From<Error> for lgn_online::server::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::BlobStorage(err) => Self::internal(err.to_string()),
            Error::Sqlx(err) => Self::internal(err.to_string()),
            Error::SqlxMigrate(err) => Self::internal(err.to_string()),
            Error::Unexpected(msg) => Self::internal(msg),
        }
    }
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
