use thiserror::Error;
use tonic::codegen::StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] StdError),
}

pub type Result<T> = std::result::Result<T, Error>;
