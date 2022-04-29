//! Content providers for various backends.

mod alias;
#[cfg(feature = "aws")]
mod aws_dynamodb;
mod cache;
mod errors;
mod grpc;
mod local;
#[cfg(feature = "lru")]
mod lru;
mod memory;
#[cfg(feature = "redis")]
mod redis;

use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::Identifier;

#[cfg(feature = "lru")]
pub use self::lru::LruAliasProvider;
#[cfg(feature = "redis")]
pub use self::redis::RedisAliasProvider;
pub use alias::Alias;
use async_trait::async_trait;
#[cfg(feature = "aws")]
pub use aws_dynamodb::AwsDynamoDbAliasProvider;
pub use cache::AliasProviderCache;
pub use errors::{Error, Result};
pub use grpc::GrpcAliasProvider;
pub use local::LocalAliasProvider;
pub use memory::MemoryAliasProvider;

/// AliasReader is a trait for read aliases to a content-store.
#[async_trait]
pub trait AliasReader: Display + Debug + Send + Sync {
    /// Returns the content-store identifier for a given alias.
    ///
    /// If no identifier is found for the specified `key` `Error::AliasNotFound`
    /// is returned.
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier>;
}

/// AliasWriter is a trait for writing aliases to a content-store.
#[async_trait]
pub trait AliasWriter: Display + Debug + Send + Sync {
    /// Registers a given alias to a content-store identifier.
    ///
    /// The caller must guarantee that the `key` is unique within the
    /// content-store space.
    ///
    /// If an alias already exists with that `key`, `Error::AliasAlreadyExists`
    /// is returned.
    #[must_use]
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier>;
}

/// `AliasProvider` is trait for all types that are both readers and writers.
pub trait AliasProvider: AliasReader + AliasWriter {}

/// Blanket implementation of `ContentProvider`.
impl<T> AliasProvider for T where T: AliasReader + AliasWriter {}

/// Blanket implementations for Arc<T> variants.

#[async_trait]
impl<T: AliasReader> AliasReader for Arc<T> {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        self.as_ref().resolve_alias(key).await
    }
}

#[async_trait]
impl<T: AliasWriter> AliasWriter for Arc<T> {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        self.as_ref().register_alias(key, id).await
    }
}

/// Blanket implementations for Box<T> variants.

#[async_trait]
impl<T: AliasReader + ?Sized> AliasReader for Box<T> {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        self.as_ref().resolve_alias(key).await
    }
}

#[async_trait]
impl<T: AliasWriter + ?Sized> AliasWriter for Box<T> {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        self.as_ref().register_alias(key, id).await
    }
}

/// Blanket implementations for &T variants.

#[async_trait]
impl<T: AliasReader + ?Sized> AliasReader for &T {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        (**self).resolve_alias(key).await
    }
}

#[async_trait]
impl<T: AliasWriter + ?Sized> AliasWriter for &T {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        (**self).register_alias(key, id).await
    }
}

#[cfg(test)]
async fn test_alias_provider(alias_provider: impl AliasProvider, key: &[u8]) -> Identifier {
    match alias_provider.resolve_alias(key).await {
        Ok(_) => panic!("expected AliasNotFound error"),
        Err(Error::AliasNotFound(err_key)) => assert_eq!(err_key.as_ref(), key),
        Err(err) => panic!("unexpected error: {}", err),
    };

    let data = b"mydata";
    let inner_id = crate::Identifier::new_data(data);
    let id = alias_provider.register_alias(key, &inner_id).await.unwrap();
    let new_id = alias_provider.resolve_alias(key).await.unwrap();

    assert_eq!(new_id, inner_id);

    id
}
