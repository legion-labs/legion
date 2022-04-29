use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, error, span_scope};

use crate::{
    traits::ContentTrackerExt, ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader,
    ContentTracker, ContentWriter, Error, Identifier, Result,
};

/// A `FallbackProvider` is a provider that stores its content in the specified
/// provider but is able to read missing values from its fallback provider.
///
/// Unlike the `CacheProvider`, values read from the fallback provider are not
/// persisted in the main provider.
#[derive(Debug, Clone)]
pub struct FallbackProvider<Inner, Fallback> {
    inner: Inner,
    fallback: Fallback,
}

impl<Inner: Display, Fallback: Display> FallbackProvider<Inner, Fallback> {
    /// Creates a new `FallbackProvider` instance who stores content in the
    /// backing remote and local providers.
    pub fn new(inner: Inner, fallback: Fallback) -> Self {
        span_scope!("FallbackProvider::new");

        debug!(
            "FallbackProvider::new(inner: {}, fallback: {})",
            inner, fallback
        );

        Self { inner, fallback }
    }
}

impl<Inner: Display, Fallback: Display> Display for FallbackProvider<Inner, Fallback> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} with read fallback by {})",
            self.inner, self.fallback
        )
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync, Fallback: ContentReader + Send + Sync> ContentReader
    for FallbackProvider<Inner, Fallback>
{
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("FallbackProvider::get_content_reader");
        debug!("FallbackProvider::get_content_reader({})", id);

        match self.inner.get_content_reader(id).await {
            Ok(reader) => {
                debug!(
                    "FallbackProvider::get_content_reader({}) -> local provider has the value",
                    id
                );

                Ok(reader)
            }
            Err(err) => {
                error!(
                    "FallbackProvider::get_content_reader({}) -> using fallback provider: {}",
                    id, err
                );

                self.fallback.get_content_reader(id).await
            }
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        async_span_scope!("FallbackProvider::get_content_readers");
        debug!("FallbackProvider::get_content_readers({:?})", ids);

        // If we can't make the request at all, try on the fallback.
        let mut readers = match self.inner.get_content_readers(ids).await {
            Ok(readers) => {
                debug!(
                    "FallbackProvider::get_content_readers({:?}) -> could query local provider for readers",
                    ids,
                );

                readers
            }
            Err(err) => {
                debug!(
                    "FallbackProvider::get_content_readers({:?}) -> could not query local provider for readers ({}): using fallback",
                    ids, err
                );

                return self.fallback.get_content_readers(ids).await;
            }
        };

        let missing_ids = readers
            .iter()
            .filter_map(|(id, reader)| {
                if let Err(Error::IdentifierNotFound(_)) = reader {
                    Some(id)
                } else {
                    None
                }
            })
            .copied()
            .cloned()
            .collect::<BTreeSet<_>>();

        if !missing_ids.is_empty() {
            debug!(
                "FallbackProvider::get_content_readers({:?}) -> creating reader on fallback for {} missing id(s)",
                ids, missing_ids.len()
            );

            readers.extend(
                self.fallback
                    .get_content_readers(&missing_ids)
                    .await?
                    .into_iter()
                    .map(|(i, reader)| (ids.get(i).unwrap(), reader)),
            );
        }

        Ok(readers)
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        async_span_scope!("FallbackProvider::resolve_alias");

        match self.inner.resolve_alias(key_space, key).await {
            Ok(id) => Ok(id),
            // If the provider fails, we just use the fallback provider.
            Err(_) => self.inner.resolve_alias(key_space, key).await,
        }
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync, Fallback: Display + Send + Sync> ContentWriter
    for FallbackProvider<Inner, Fallback>
{
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("FallbackProvider::get_content_writer");

        self.inner.get_content_writer(id).await
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("FallbackProvider::register_alias");

        self.inner.register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<Inner: ContentTracker + Send + Sync, Fallback: ContentReader + Send + Sync> ContentTracker
    for FallbackProvider<Inner, Fallback>
{
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        async_span_scope!("FallbackProvider::remove_content");

        self.inner.remove_content(id).await
    }

    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        async_span_scope!("FallbackProvider::pop_referenced_identifiers");

        self.inner.pop_referenced_identifiers().await
    }
}

impl<Inner: ContentTracker + Send + Sync, Fallback: ContentWriter + Send + Sync>
    FallbackProvider<Inner, Fallback>
{
    /// Pop all the referenced identifiers from the inner provider and copy them
    /// to the fallback provider.
    ///
    /// # Errors
    ///
    /// If the copy fails after some elements have been copied,
    /// `Error::CopyInterrupted` is returned which contains the list of
    /// identifiers that have not been copied.
    pub async fn pop_referenced_identifiers_and_copy(&self) -> Result<()> {
        async_span_scope!("FallbackProvider::pop_referenced_identifiers_and_copy");

        self.inner
            .pop_referenced_identifiers_and_copy_to(&self.fallback)
            .await
    }
}
