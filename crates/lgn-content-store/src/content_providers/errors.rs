use thiserror::Error;

use super::{HashRef, InvalidHashRef};

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("the provider does not support unwritting")]
    UnwriteNotSupported,
    #[error("the hash reference `{0}` was not found")]
    HashRefNotFound(HashRef),
    #[error("the hash reference `{0}` already exists")]
    HashRefAlreadyExists(HashRef),
    #[error(
        "the data sent should have the hash reference `{expected}` but it has `{actual}` instead"
    )]
    UnexpectedHashRef { expected: HashRef, actual: HashRef },
    #[error("the data for hash reference `{0}` is corrupted")]
    CorruptedHashRef(HashRef),
    #[error("invalid hash reference: {0}")]
    InvalidHashRef(#[from] InvalidHashRef),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
