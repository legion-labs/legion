use axum::response::IntoResponse;
use http::StatusCode;
use thiserror::Error;

use crate::types::{PermissionId, SpaceId, UserId};

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migrate: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("AWS Cognito: {0}")]
    AwsCognito(#[from] aws_sdk_cognitoidentityprovider::Error),
    #[error("types: {0}")]
    Types(#[from] crate::types::Error),
    #[error("the authentication info is missing")]
    Unauthorized,
    #[error("user `{0}` does not have the `{1}` permission in {}", .2.as_ref().map_or_else(|| "global space".to_string(), |s| format!("space `{}`", s)))]
    PermissionDenied(UserId, PermissionId, Option<SpaceId>),
    #[error("configuration: {0}")]
    Configuration(String),
    #[error("unexpected: {0}")]
    Unexpected(String),
}

impl From<Error> for lgn_online::server::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(err) => Self::Internal(err.to_string()),
            Error::SqlxMigrate(err) => Self::Internal(err.to_string()),
            Error::AwsCognito(err) => Self::Internal(err.to_string()),
            Error::Types(err) => Self::Internal(err.to_string()),
            Error::Unauthorized => Self::Custom(
                (StatusCode::UNAUTHORIZED, "missing authentication info").into_response(),
            ),
            Error::PermissionDenied(user_id, permission_id, space_id) => Self::Custom(
                match space_id {
                    Some(space_id) => (
                        StatusCode::FORBIDDEN,
                        format!(
                            "user `{}` does not have the `{}` permission in space `{}`",
                            user_id, permission_id, space_id
                        ),
                    ),
                    None => (
                        StatusCode::FORBIDDEN,
                        format!(
                            "user `{}` does not have the global `{}` permission",
                            user_id, permission_id
                        ),
                    ),
                }
                .into_response(),
            ),
            Error::Configuration(msg) | Error::Unexpected(msg) => Self::Internal(msg),
        }
    }
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
