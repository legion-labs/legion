use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_yaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
