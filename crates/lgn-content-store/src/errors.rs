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
    #[error("the content is not referenced by the tracker: {0}")]
    IdentifierNotReferenced(Identifier),
    #[error("the content already exists: {0}")]
    IdentifierAlreadyExists(Identifier),
    #[error("the alias was not found: {key_space}/{key}")]
    AliasNotFound { key_space: String, key: String },
    #[error("the alias already exists: {key_space}/{key}")]
    AliasAlreadyExists { key_space: String, key: String },
    #[error("the copy was interrupted for identifier {id} ({} identifiers remaining): {err}", identifiers.len())]
    CopyInterrupted {
        id: Identifier,
        identifiers: Vec<Identifier>,
        err: Box<Error>,
    },
    #[error("the content is corrupted: {0}")]
    Corrupt(Identifier),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde decode: {0}")]
    SerdeDecode(#[from] rmp_serde::decode::Error),
    #[error("hex decode: {0}")]
    HexDecode(#[from] hex::FromHexError),
    #[error("unsupported index operation: {0}")]
    UnsupportedIndexOperation(String),
    #[error("invalid index key: {0}")]
    InvalidIndexKey(String),
    #[error("corrupted tree: {0}")]
    CorruptedTree(String),
    #[error("index tree leaf node was not found at `{0}`")]
    IndexTreeLeafNodeNotFound(crate::indexing::IndexKey),
    #[error("index tree leaf node already exists at `{0}`: {1:?}")]
    IndexTreeLeafNodeAlreadyExists(crate::indexing::IndexKey, crate::indexing::TreeLeafNode),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
