use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use async_trait::async_trait;

use crate::{
    traits::get_content_readers_impl, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result,
};

/// A `SmallContentProvider` is a provider that implements the small-content optimization or delegates to a specified provider.
#[derive(Debug)]
pub struct SmallContentProvider<Inner> {
    inner: Inner,
}

impl<Inner: Clone> Clone for SmallContentProvider<Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Inner> SmallContentProvider<Inner> {
    /// Instantiate a new small content provider that wraps the specified
    /// provider using the default identifier size threshold.
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for SmallContentProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        if let Identifier::Data(data) = id {
            Ok(Box::pin(std::io::Cursor::new(data.to_vec())))
        } else {
            self.inner.get_content_reader(id).await
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        // Always forward to the inner provider.
        self.inner.resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for SmallContentProvider<Inner> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        if id.is_data() {
            return Err(Error::AlreadyExists);
        }

        self.inner.get_content_writer(id).await
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        // Always forward to the inner provider.
        self.inner.register_alias(key_space, key, id).await
    }
}
