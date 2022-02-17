use thiserror::Error;

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(#[source] anyhow::Error),
    #[error("data mismatch: {reason}")]
    DataMismatch { reason: String },
    #[error("invalid hash algorithm")]
    InvalidHashAlgorithm,
    #[error("the content was not found")]
    NotFound,
    #[error("the content already exists")]
    AlreadyExists,
    #[error("the content is corrupted")]
    Corrupt,
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
