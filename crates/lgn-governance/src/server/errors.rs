use thiserror::Error;

use crate::types::{PermissionId, SpaceId, UserId};

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("sqlx migrate: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("an identical value already exists in the database")]
    AlreadyExists,
    #[error("the value does not exist in the database")]
    DoesNotExist,
    #[error("the operation could not complete because a conflict arose")]
    Conflict,
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

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(err) => {
                if matches!(err.code(), Some(c) if "23000".eq(&c)) {
                    return Self::AlreadyExists;
                }
            }
            sqlx::Error::RowNotFound => return Self::DoesNotExist,
            _ => {}
        }

        Self::Sqlx(err)
    }
}

impl From<Error> for lgn_online::server::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(err) => Self::internal(err.to_string()),
            Error::SqlxMigrate(err) => Self::internal(err.to_string()),
            Error::AlreadyExists => {
                Self::conflict("an identical value already exists in the database")
            }
            Error::DoesNotExist => Self::not_found("the value does not exist in the database"),
            Error::Conflict => {
                Self::conflict("the operation could not complete because a conflict arose")
            }
            Error::AwsCognito(err) => Self::internal(err.to_string()),
            Error::Types(err) => Self::internal(err.to_string()),
            Error::Unauthorized => Self::unauthorized("missing authentication info"),
            Error::PermissionDenied(user_id, permission_id, space_id) => {
                Self::forbidden(match space_id {
                    Some(space_id) => format!(
                        "user `{}` does not have the `{}` permission in space `{}`",
                        user_id, permission_id, space_id
                    ),
                    None => format!(
                        "user `{}` does not have the global `{}` permission",
                        user_id, permission_id
                    ),
                })
            }
            Error::Configuration(msg) | Error::Unexpected(msg) => Self::internal(msg),
        }
    }
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
