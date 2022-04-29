use thiserror::Error;

use super::Alias;

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("the alias `{0}` was not found")]
    AliasNotFound(Alias),
    #[error("the alias `{0}` already exists")]
    AliasAlreadyExists(Alias),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(#[from] crate::InvalidIdentifier),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
