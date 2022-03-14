use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;

use crate::{
    AliasRegisterer, AliasResolver, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Identifier, Result,
};

/// A struct that makes it easy to create a `ContentProvider` that is also an `AliasProvider`.
pub struct Aliaser<A, P> {
    aliaser: A,
    provider: P,
}

impl<A, P> Aliaser<A, P> {
    pub fn new(aliaser: A, provider: P) -> Self {
        Self { aliaser, provider }
    }
}

#[async_trait]
impl<A: AliasResolver + Send + Sync, P: Send + Sync> AliasResolver for Aliaser<A, P> {
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.aliaser.resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<A: AliasRegisterer + Send + Sync, P: Send + Sync> AliasRegisterer for Aliaser<A, P> {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.aliaser.register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<A: Send + Sync, P: ContentReader + Send + Sync> ContentReader for Aliaser<A, P> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        self.provider.get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        self.provider.get_content_readers(ids).await
    }
}

#[async_trait]
impl<A: Send + Sync, P: ContentWriter + Send + Sync> ContentWriter for Aliaser<A, P> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.provider.get_content_writer(id).await
    }
}
