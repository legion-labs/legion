use thiserror::Error;
use tonic::codegen::StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to run server: {0}")]
    RunServerFailure(String),
    #[error(transparent)]
    Other(#[from] StdError),
}

pub type Result<T> = std::result::Result<T, Error>;
