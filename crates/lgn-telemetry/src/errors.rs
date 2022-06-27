use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("prost decode: {0}")]
    ProstDecode(#[from] prost::DecodeError),
    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
