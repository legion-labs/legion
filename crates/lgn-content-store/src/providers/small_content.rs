use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
};

use async_trait::async_trait;
use lgn_tracing::debug;

use crate::{
    traits::{get_content_readers_impl, WithOrigin},
    ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader, ContentTracker, ContentWriter,
    Error, Identifier, Origin, Result,
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

impl<Inner: Display> Display for SmallContentProvider<Inner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} + small-content optimization", self.inner)
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for SmallContentProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        if let Identifier::Data(data) = id {
            debug!("SmallContentProvider::get_content_reader({}) -> returning data contained in the identifier", id);

            Ok(std::io::Cursor::new(data.to_vec()).with_origin(Origin::InIdentifier {}))
        } else {
            debug!(
                "SmallContentProvider::get_content_reader({}) -> calling the inner provider",
                id
            );

            self.inner.get_content_reader(id).await
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
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
            return Err(Error::IdentifierAlreadyExists(id.clone()));
        }

        self.inner.get_content_writer(id).await
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        // Always forward to the inner provider.
        self.inner.register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<Inner: ContentTracker + Send + Sync> ContentTracker for SmallContentProvider<Inner> {
    /// Decrease the reference count of the specified content.
    ///
    /// If the reference count is already zero, the call is a no-op and no error
    /// is returned.
    ///
    /// # Errors
    ///
    /// If the content was not referenced, `Error::IdentifierNotReferenced` is
    /// returned.
    ///
    /// This can happen when the content was existing prior to the tracker being
    /// instanciated, which is a very common case. This is not an error and
    /// callers should be prepared to handle this.
    ///
    /// To handle this case, callers can use the
    /// `ContentTrackerExt::try_remove_content` method instead which only
    /// returns fatal errors.
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        if id.is_data() {
            return Err(Error::IdentifierNotReferenced(id.clone()));
        }

        self.inner.remove_content(id).await
    }

    /// Gets all the identifiers with a stricly positive reference count.
    ///
    /// The reference counts of the returned identifiers are guaranteed to be
    /// zero after the call.
    ///
    /// The typical use-case for this method is to synchronize all the returned
    /// identifiers to a more persistent content-store instance.
    ///
    /// This can be achieved more simply by calling the
    /// `pop_referenced_identifiers_and_copy_to` method from the
    /// `ContentTrackerExt` trait.
    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        self.inner.pop_referenced_identifiers().await
    }
}
