use thiserror::Error;

use crate::{Identifier, InvalidIdentifier, InvalidManifest};

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("missing `content_store.{section}` section in configuration")]
    MissingConfigurationSection { section: String },
    #[error("configuration error: {0}")]
    Configuration(#[from] lgn_config::Error),
    #[error("online error: {0}")]
    Online(#[from] lgn_online::Error),
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(#[from] InvalidIdentifier),
    #[error("invalid manifest: {0}")]
    InvalidManifest(#[from] InvalidManifest),
    #[error("identifier already exists: {0}")]
    IdentifierAlreadyExists(Identifier),
    #[error("identifier not found: {0}")]
    IdentifierNotFound(Identifier),
    #[error("content provider error: {0}")]
    ContentProvider(crate::content_providers::Error),
    #[error("alias provider error: {0}")]
    AliasProvider(crate::alias_providers::Error),
    #[error("indexing error: {0}")]
    Indexing(#[from] crate::indexing::Error),
    #[error("invalid data-space: {0}")]
    InvalidDataSpace(String),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl From<crate::content_providers::Error> for Error {
    fn from(err: crate::content_providers::Error) -> Self {
        match err {
            crate::content_providers::Error::HashRefAlreadyExists(id) => {
                Self::IdentifierAlreadyExists(Identifier::new_hash_ref(id))
            }
            crate::content_providers::Error::HashRefNotFound(id) => {
                Self::IdentifierNotFound(Identifier::new_hash_ref(id))
            }
            err => Self::ContentProvider(err),
        }
    }
}

impl From<crate::alias_providers::Error> for Error {
    fn from(err: crate::alias_providers::Error) -> Self {
        match err {
            crate::alias_providers::Error::AliasAlreadyExists(id) => {
                Self::IdentifierAlreadyExists(Identifier::new_alias(id))
            }
            crate::alias_providers::Error::AliasNotFound(id) => {
                Self::IdentifierNotFound(Identifier::new_alias(id))
            }
            err => Self::AliasProvider(err),
        }
    }
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
