use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("types: {0}")]
    Types(#[from] crate::types::Error),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
