use thiserror::Error;
use tonic::codegen::StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid authorization URL: {0}")]
    InvalidAuthorizationUrl(String),
    #[error("internal server error: {0}")]
    InternalServer(hyper::Error),
    #[error("failed to execute the interactive login process: {0}")]
    InteractiveProcess(std::io::Error),
    #[error("internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Other(#[from] StdError),
}

impl Error {
    #[doc = "Automatically converts an [`std::error::Error`] into an [`Error::Internal`]"]
    pub fn internal(error: impl std::error::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
