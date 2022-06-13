use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("the client is not authorized to perform the requested operation")]
    Unauthorized,
    #[error("the stack was already initialized")]
    StackAlreadyInitialized,
    #[error("server error: {0}")]
    ServerError(#[from] lgn_online::server::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
