use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use lgn_tracing::error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("internal: {0}")]
    Internal(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("custom")]
    Custom(Response),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::Internal(format!("hyper: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(format!("serde json: {}", err))
    }
}

impl From<serde_qs::Error> for Error {
    fn from(err: serde_qs::Error) -> Self {
        Self::Internal(format!("serde qs: {}", err))
    }
}

impl From<crate::codegen::encoding::Error> for Error {
    fn from(err: crate::codegen::encoding::Error) -> Self {
        Self::Internal(format!("encoding: {}", err))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Internal(msg) => {
                error!("returning error as internal server error response: {}", msg);

                // Let's be careful and *NOT* return the message as the body in
                // this case as it could contain sensitive information.
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            Self::Custom(response) => response,
        }
    }
}

pub trait ErrorExt<T>: Sized {
    /// Converts to a server error.
    fn into_server_error(self, status_code: StatusCode) -> Result<T, Error>;

    /// Converts to an internal server error.
    fn into_internal_server_error(self) -> Result<T, Error> {
        self.into_server_error(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Converts to a bad request error.
    fn into_bad_request(self) -> Result<T, Error> {
        self.into_server_error(StatusCode::BAD_REQUEST)
    }
}

impl<T, E: std::error::Error> ErrorExt<T> for Result<T, E> {
    fn into_server_error(self, status_code: StatusCode) -> Result<T, Error> {
        self.map_err(|err| match status_code {
            StatusCode::INTERNAL_SERVER_ERROR => Error::Internal(err.to_string()),
            StatusCode::BAD_REQUEST => Error::BadRequest(err.to_string()),
            _ => Error::Custom((status_code, err.to_string()).into_response()),
        })
    }
}
