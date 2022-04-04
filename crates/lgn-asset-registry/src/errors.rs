use thiserror::Error;

/// An error type for the asset registry crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("content store error: {0}")]
    ContentStore(#[from] lgn_content_store2::Error),
    #[error("serde deserialization error: {0}")]
    SerdeJSON(#[from] serde_json::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
