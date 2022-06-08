use thiserror::Error;

/// An error type.
#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid space id: {0}")]
    InvalidSpaceId(String),
    #[error("invalid permission id: {0}")]
    InvalidPermissionId(String),
    #[error("invalid role id: {0}")]
    InvalidRoleId(String),
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
}

/// A result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
