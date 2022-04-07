use thiserror::Error;

use crate::Identifier;

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
    InvalidIdentifier(#[source] anyhow::Error),
    #[error("data mismatch: {reason}")]
    DataMismatch { reason: String },
    #[error("invalid hash algorithm")]
    InvalidHashAlgorithm,
    #[error("invalid chunk index")]
    InvalidChunkIndex(#[source] anyhow::Error),
    #[error("invalid data space: {0}")]
    InvalidDataSpace(String),
    #[error("the content was not found: {0}")]
    IdentifierNotFound(Identifier),
    #[error("the content already exists: {0}")]
    IdentifierAlreadyExists(Identifier),
    #[error("the alias was not found: {key_space}/{key}")]
    AliasNotFound { key_space: String, key: String },
    #[error("the alias already exists: {key_space}/{key}")]
    AliasAlreadyExists { key_space: String, key: String },
    #[error("the content is corrupted: {0}")]
    Corrupt(Identifier),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
