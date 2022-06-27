use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("client error: {0}")]
    Client(#[from] lgn_online::client::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
