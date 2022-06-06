use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid permission id: {0}")]
    InvalidPermissionId(String),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
