//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

mod buf_utils;
mod chunk_identifier;
mod chunker;
mod config;
mod data_space;
mod errors;
mod identifier;
mod providers;
mod traits;

pub use chunk_identifier::ChunkIdentifier;
pub use chunker::{ChunkIndex, Chunker};
pub use config::{
    AddressProviderConfig, AwsDynamoDbProviderConfig, AwsS3ProviderConfig, Config,
    LocalProviderConfig, LruProviderConfig, ProviderConfig, RedisProviderConfig,
};
pub use data_space::DataSpace;
pub use errors::{Error, Result};
pub use identifier::{HashAlgorithm, Identifier};
pub use providers::*;
pub use traits::{
    ContentAddressProvider, ContentAddressReader, ContentAddressWriter, ContentAsyncRead,
    ContentAsyncWrite, ContentProvider, ContentReader, ContentReaderExt, ContentWriter,
    ContentWriterExt,
};
