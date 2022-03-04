use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("content-store error: {0}")]
    ContentStore(#[from] lgn_content_store2::Error),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
