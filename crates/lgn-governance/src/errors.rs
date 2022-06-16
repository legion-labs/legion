use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("types: {0}")]
    Types(#[from] crate::types::Error),
    #[error("config: {0}")]
    Config(#[from] lgn_config::Error),
    #[error("online: {0}")]
    Online(#[from] lgn_online::Error),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
