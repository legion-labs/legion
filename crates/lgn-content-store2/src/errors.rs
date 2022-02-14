use thiserror::Error;

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(#[source] anyhow::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T> = std::result::Result<T, Error>;
