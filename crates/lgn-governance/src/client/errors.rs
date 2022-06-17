use thiserror::Error;

use crate::types::{ExtendedUserId, Space, SpaceId, UserAlias};

/// An error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("the client is not authorized to perform the requested operation")]
    Unauthorized,
    #[error("client error: {0}")]
    ClientError(#[from] lgn_online::client::Error),
    #[error("types: {0}")]
    Types(#[from] crate::types::Error),
    #[error("the stack was already initialized")]
    StackAlreadyInitialized,
    #[error("a space with the id `{}` already exists", .0.id)]
    SpaceAlreadyExists(Space),
    #[error("no space exists with the id `{0}`")]
    SpaceDoesNotExist(SpaceId),
    #[error("the operation cannot be attempted while the space is being used")]
    SpaceInUse(Space),
    #[error("the user `{0}` was not found")]
    UserNotFound(ExtendedUserId),
    #[error("the user alias `{0}` is already registered")]
    UserAliasAlreadyExists(UserAlias),
    #[error("the user alias `{0}` does not exist")]
    UserAliasNotFound(UserAlias),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
