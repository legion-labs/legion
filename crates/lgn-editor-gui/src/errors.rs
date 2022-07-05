pub trait FromReqwestError {
    fn from_reqwest_error(reqwest_error: reqwest::Error) -> Self;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
pub enum Error {
    #[error("Reqwest error {0}")]
    Reqwest(String),
    #[error("Dom error {0}")]
    Js(String),
    #[error("Auth error {0}")]
    Auth(String),
}

impl FromReqwestError for Error {
    fn from_reqwest_error(reqwest_error: reqwest::Error) -> Self {
        Self::Reqwest(reqwest_error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
