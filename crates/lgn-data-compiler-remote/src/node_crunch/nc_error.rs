//! This module contains the common error type for server and node.

use std::{io, net, sync};

use thiserror::Error;
use url::ParseError;

use crate::node_crunch::nc_node_info::NodeID;

/// This data structure contains all error codes for the server and the nodes.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum NCError {
    /// Parsing the IP address went wrong.
    #[error("IP address parse error: {0}")]
    IPAddrParse(#[from] net::AddrParseError),
    /// Common IO error, usually network related.
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    /// Data could not be serialized for sending over the network.
    #[error("Serialize bincode error: {0}")]
    Serialize(bincode::Error),
    /// Data coming from the network could not be deserialized.
    #[error("Deserialize bincode error: {0}")]
    Deserialize(bincode::Error),
    /// The [`bincode`] crate has its own error.
    #[error("Bincode error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
    /// Decompression error
    #[error("Decompression error")]
    Decompress(#[from] lz4_flex::block::DecompressError),
    /// Encrypt error
    #[error("Encrypt error")]
    Encrypt,
    /// Decrypt error
    #[error("Decrypt error")]
    Decrypt,
    /// The node expected a specific message from the server but got s.th. totally different.
    #[error("Server message mismatch error")]
    ServerMsgMismatch,
    /// The server expected a specific message from the node but got s.th. totally different.
    #[error("Node message mismatch error")]
    NodeMsgMismatch,
    /// A different node id was expected. Expected first node id, found second node id.
    #[error("Node id mismatch error, couldn't find job for {0}")]
    NodeIDMismatch(NodeID),
    /// [`Mutex`](std::sync::Mutex) could not be locked or a thread did panic while holding the lock.
    #[error("Mutex poisson error")]
    MutexPoison,
    #[error("URL parse error: {0}")]
    Url(#[from] ParseError),
    #[error("CAS error: {0}")]
    CASError(#[from] lgn_content_store::Error),
    #[error("Compiler error: {0}")]
    CompilerError(#[from] lgn_data_compiler::compiler_api::CompilerError),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::error::Error),
}

impl<T> From<sync::PoisonError<sync::MutexGuard<'_, T>>> for NCError {
    fn from(_: sync::PoisonError<sync::MutexGuard<'_, T>>) -> Self {
        Self::MutexPoison
    }
}
