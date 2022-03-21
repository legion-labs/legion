use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("config error: {0}")]
    Config(#[from] figment::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
