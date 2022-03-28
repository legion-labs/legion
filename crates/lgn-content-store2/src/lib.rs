//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

mod buf_utils;
mod chunk_identifier;
mod chunker;
mod config;
mod errors;
mod identifier;
mod providers;
mod traits;

pub use chunk_identifier::ChunkIdentifier;
pub use chunker::{ChunkIndex, Chunker};
pub use config::{
    AwsS3ProviderConfig, Config, GrpcProviderConfig, LocalProviderConfig, ProviderConfig,
    RedisProviderConfig,
};
pub use errors::{Error, Result};
pub use identifier::{HashAlgorithm, Identifier};
pub use providers::*;
pub use traits::{
    AliasContentReaderExt, AliasContentWriterExt, AliasProvider, AliasRegisterer, AliasResolver,
    ContentAddressProvider, ContentAddressReader, ContentAddressWriter, ContentAsyncRead,
    ContentAsyncWrite, ContentProvider, ContentReader, ContentReaderExt, ContentWriter,
    ContentWriterExt,
};
