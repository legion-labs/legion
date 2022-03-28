use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("authentication error: {0}")]
    Authentication(#[from] crate::authentication::Error),
    #[error("configuration error: {0}")]
    Config(#[from] lgn_config::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
