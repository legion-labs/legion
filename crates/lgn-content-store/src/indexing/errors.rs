use std::string::FromUtf8Error;

use thiserror::Error;

use super::{IndexKey, TreeLeafNode};

/// An error type for the content-store crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("corrupted tree: {0}")]
    CorruptedTree(String),
    #[error("invalid index key: {0}")]
    InvalidIndexKey(String),
    #[error("invalid index key display format: {0}")]
    InvalidIndexKeyDisplayFormat(String),
    #[error("the index tree leaf node already exists at `{0:?}`")]
    IndexTreeLeafNodeAlreadyExists(IndexKey, TreeLeafNode),
    #[error("the index tree leaf node wasn't found at `{0:?}`")]
    IndexTreeLeafNodeNotFound(IndexKey),
    #[error("the index tree node wasn't found at `{0:?}`")]
    IndexTreeNodeNotFound(IndexKey),
    #[error("invalid tree identifier: {0}")]
    InvalidTreeIdentifier(crate::InvalidIdentifier),
    #[error("invalid index identifier: {0}")]
    InvalidIndexIdentifier(crate::InvalidIdentifier),
    #[error("invalid resource identifier: {0}")]
    InvalidResourceIdentifier(crate::InvalidIdentifier),
    #[error("invalid indexer identifier: {0}")]
    InvalidIndexerIdentifier(crate::InvalidIdentifier),
    #[error("{0}")]
    ContentProvider(Box<crate::Error>),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hex error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("MessagePack error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        Self::ContentProvider(Box::new(err))
    }
}

/// A result type that can be used to indicate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;
