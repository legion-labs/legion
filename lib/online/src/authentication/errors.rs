use thiserror::Error;
use tonic::codegen::StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid authorization URL: {0}")]
    InvalidAuthorizationUrl(String),
    #[error("internal server error: {0}")]
    InternalServerError(hyper::Error),
    #[error("failed to execute the interactive login process: {0}")]
    InteractiveProcessError(std::io::Error),
    #[error("internal error: {0}")]
    InternalError(String),
    #[error(transparent)]
    Other(#[from] StdError),
}

pub type Result<T> = std::result::Result<T, Error>;
