use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use futures::future::join_all;
use lgn_tracing::async_span_scope;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, Identifier, Origin, Result};

/// A trait that extends `AsyncRead` with an origin.
pub trait AsyncReadWithOrigin: AsyncRead {
    fn origin(&self) -> &Origin;
}

impl<T: AsyncReadWithOrigin> AsyncReadWithOrigin for Pin<Box<T>> {
    fn origin(&self) -> &Origin {
        (**self).origin()
    }
}

#[pin_project]
struct AsyncReadWithOriginImpl<R: AsyncRead> {
    #[pin]
    inner: R,
    origin: Origin,
}

impl<R: AsyncRead> AsyncReadWithOriginImpl<R> {
    pub(crate) fn new(inner: R, origin: Origin) -> Self {
        Self { inner, origin }
    }
}

impl<R: AsyncRead> AsyncRead for AsyncReadWithOriginImpl<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<R: AsyncRead> AsyncReadWithOrigin for AsyncReadWithOriginImpl<R> {
    fn origin(&self) -> &Origin {
        &self.origin
    }
}

pub(crate) trait WithOrigin {
    fn with_origin(self, origin: Origin) -> ContentAsyncReadWithOrigin;
}

impl<R: AsyncRead + Send + Sync + 'static> WithOrigin for R {
    fn with_origin(self, origin: Origin) -> ContentAsyncReadWithOrigin {
        Box::pin(AsyncReadWithOriginImpl::new(self, origin))
    }
}

/// A reader as returned by the `ContentReader` trait.
pub type ContentAsyncReadWithOrigin = Pin<Box<dyn AsyncReadWithOrigin + Send>>;

/// A reader as returned by a chunker.
pub type ContentAsyncRead = Pin<Box<dyn AsyncRead + Send>>;

/// A writer as returned by the `ContentWriter` trait.
pub type ContentAsyncWrite = Pin<Box<dyn AsyncWrite + Send>>;

/// ContentReader is a trait for reading content from a content-store.
#[async_trait]
pub trait ContentReader: Display {
    /// Returns an async reader that reads the content referenced by the
    /// specified identifier.
    ///
    /// If the identifier does not match any content, `Error::IdentifierNotFound` is
    /// returned.
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin>;

    /// Returns an async reader for each of the specified identifiers.
    ///
    /// If the content for a given identifier does not exist, `Error::IdentifierNotFound`
    /// is returned instead.
    ///
    /// If the high-level request fails, an error is returned.
    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>>;

    /// Returns the content-store identifier for a given alias.
    ///
    /// If no identifier is found for the specified `key` in the specified
    /// `key_space`, `Error::AliasNotFound` is returned.
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier>;
}

/// A default implementation for the `get_content_readers` method that just
/// calls in parallel `get_content_reader` for each identifier.
pub(crate) async fn get_content_readers_impl<'ids>(
    reader: &(dyn ContentReader + Send + Sync),
    ids: &'ids BTreeSet<Identifier>,
) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
    let futures = ids
        .iter()
        .map(|id| async move { (id, reader.get_content_reader(id).await) })
        .collect::<Vec<_>>();

    Ok(join_all(futures).await.into_iter().collect())
}

#[async_trait]
pub trait ContentReaderExt: ContentReader {
    /// Check whether the identifier exists in the store.
    async fn exists(&self, id: &Identifier) -> bool {
        async_span_scope!("ContentReaderExt::exists");

        if let Identifier::Data(_) = id {
            return true;
        }

        self.get_content_reader(id).await.is_ok()
    }

    /// Read the origin for a given identifier.
    async fn read_origin(&self, id: &Identifier) -> Result<Origin> {
        async_span_scope!("ContentReaderExt::read_origin");

        if let Identifier::Data(_) = id {
            return Ok(Origin::InIdentifier {});
        }

        self.get_content_reader(id)
            .await
            .map(|r| r.origin().clone())
    }

    /// Read the content referenced by the specified identifier with its origin.
    async fn read_content_with_origin(&self, id: &Identifier) -> Result<(Vec<u8>, Origin)> {
        async_span_scope!("ContentReaderExt::read_content_with_origin");

        if let Identifier::Data(data) = id {
            return Ok((data.to_vec(), Origin::InIdentifier {}));
        }

        let mut reader = self.get_content_reader(id).await?;

        let mut result = Vec::with_capacity(id.data_size());

        reader
            .read_to_end(&mut result)
            .await
            .map_err(|err| anyhow::anyhow!("failed to read content: {}", err).into())
            .map(|_| (result, reader.origin().clone()))
    }

    /// Read the content referenced by the specified identifier.
    async fn read_content(&self, id: &Identifier) -> Result<Vec<u8>> {
        async_span_scope!("ContentReaderExt::read_content");

        self.read_content_with_origin(id)
            .await
            .map(|(data, _)| data)
    }

    /// Read the contents referenced by the specified identifiers.
    async fn read_contents<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<Vec<u8>>>> {
        async_span_scope!("ContentReaderExt::read_contents");

        let readers = self.get_content_readers(ids).await?;
        let futures = readers
            .into_iter()
            .map(|(id, r)| async move {
                (
                    id,
                    match r {
                        Ok(mut reader) => {
                            let mut result = Vec::with_capacity(id.data_size());
                            reader
                                .read_to_end(&mut result)
                                .await
                                .map_err(|err| {
                                    anyhow::anyhow!("failed to read content for `{}`: {}", id, err)
                                        .into()
                                })
                                .map(|_| result)
                        }
                        Err(err) => Err(err),
                    },
                )
            })
            .collect::<Vec<_>>();

        Ok(join_all(futures).await.into_iter().collect())
    }

    /// Read the content referenced by the specified identifier.
    async fn get_alias_reader(
        &self,
        key_space: &str,
        key: &str,
    ) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("ContentReaderExt::get_alias_reader");

        let id = self.resolve_alias(key_space, key).await?;

        self.get_content_reader(&id).await
    }

    /// Read the content referenced by the specified identifier.
    async fn read_alias(&self, key_space: &str, key: &str) -> Result<Vec<u8>> {
        async_span_scope!("ContentReaderExt::read_alias");

        let id = self.resolve_alias(key_space, key).await?;

        self.read_content(&id).await
    }

    /// Copy all the specified identifiers to the specified `ContentWriter`.
    ///
    /// # Errors
    ///
    /// If the copy fails, `Error::CopyInterrupted` is returned which contains
    /// the list of identifiers that could not be copied.
    async fn copy_to(
        &self,
        content_writer: impl ContentWriter + Send + Sync + 'async_trait,
        identifiers: Vec<Identifier>,
    ) -> Result<()> {
        // We must collect all the identifiers cause we can't have `identifiers`
        // living across an `await`.
        let mut identifiers = identifiers.into_iter();

        while let Some(id) = identifiers.next() {
            match content_writer.get_content_writer(&id).await {
                Ok(mut writer) => {
                    let mut reader = match self.get_content_reader(&id).await {
                        Ok(reader) => reader,
                        Err(err) => {
                            return Err(Error::CopyInterrupted {
                                id,
                                identifiers: identifiers.collect(),
                                err: Box::new(err),
                            })
                        }
                    };

                    if let Err(err) = tokio::io::copy(&mut reader, &mut writer).await {
                        return Err(Error::CopyInterrupted {
                            id,
                            identifiers: identifiers.collect(),
                            err: Box::new(err.into()),
                        });
                    }

                    // Don't forget to shut down the writer or the write will
                    // actually never happen!
                    if let Err(err) = writer.shutdown().await {
                        return Err(Error::CopyInterrupted {
                            id,
                            identifiers: identifiers.collect(),
                            err: Box::new(err.into()),
                        });
                    }
                }
                Err(Error::IdentifierAlreadyExists(_)) => {}
                Err(err) => {
                    return Err(Error::CopyInterrupted {
                        id,
                        identifiers: identifiers.collect(),
                        err: Box::new(err),
                    })
                }
            }
        }

        Ok(())
    }
}

impl<T: ContentReader> ContentReaderExt for T {}

/// ContentWriter is a trait for writing content to a content-store.
#[async_trait]
pub trait ContentWriter: Display {
    /// Returns an async write to which the content referenced by the specified
    /// specified identifier can be written.
    ///
    /// # Note
    ///
    /// The caller is responsible for writing exactly the data that matches with
    /// the identifier.
    ///
    /// This call in only intended for low-level efficient operations and very
    /// error prone. If possible, use the `write_content` method instead.
    ///
    /// # Errors
    ///
    /// If the data already exists, `Error::IdentifierAlreadyExists` is returned and the
    /// caller should consider that the write operation is not necessary.
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite>;

    /// Registers a given alias to a content-store identifier.
    ///
    /// The caller must guarantee that the `key` is unique within the specified
    /// `key_space`.
    ///
    /// If an alias already exists with that `key` and `key_space`, `Error::AliasAlreadyExists`
    /// is returned.
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()>;
}

#[async_trait]
pub trait ContentWriterExt: ContentWriter {
    /// Write the specified content and returns the newly associated identifier.
    ///
    /// If the content already exists, the write is a no-op and no error is
    /// returned.
    async fn write_content(&self, data: &[u8]) -> Result<Identifier> {
        async_span_scope!("ContentReaderExt::write_content");

        let id = Identifier::new(data);

        if id.is_data() {
            return Ok(id);
        }

        let mut writer = match self.get_content_writer(&id).await {
            Ok(writer) => writer,
            Err(Error::IdentifierAlreadyExists(_)) => return Ok(id),
            Err(err) => return Err(err),
        };

        writer
            .write_all(data)
            .await
            .map_err(|err| anyhow::anyhow!("failed to write content: {}", err))?;

        writer
            .shutdown()
            .await
            .map_err(|err| anyhow::anyhow!("failed to flush content: {}", err).into())
            .map(|_| id)
    }

    /// Get a write for the content referenced by the specified identifier.
    async fn write_alias(&self, key_space: &str, key: &str, data: &[u8]) -> Result<Identifier> {
        async_span_scope!("ContentReaderExt::write_alias");

        let id = self.write_content(data).await?;

        match self.register_alias(key_space, key, &id).await {
            Ok(_) | Err(Error::AliasAlreadyExists { .. }) => Ok(id),
            Err(err) => Err(err),
        }
    }
}

impl<T: ContentWriter> ContentWriterExt for T {}

/// `ContentProvider` is trait for all types that are both readers and writers.
pub trait ContentProvider: ContentReader + ContentWriter {}

/// Blanket implementation of `ContentProvider`.
impl<T> ContentProvider for T where T: ContentReader + ContentWriter {}

/// ContentTracker is a trait for tracking content.
#[async_trait]
pub trait ContentTracker: ContentProvider {
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
    async fn remove_content(&self, id: &Identifier) -> Result<()>;

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
    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>>;
}

#[async_trait]
pub trait ContentTrackerExt: ContentTracker + ContentReaderExt {
    /// Remove the content without returning an error if the content was not
    /// referenced before.
    ///
    /// # Errors
    ///
    /// Any other error than `Error::IdentifierNotReferenced` is returned.
    async fn try_remove_content(&self, id: &Identifier) -> Result<()> {
        match self.remove_content(id).await {
            Ok(()) | Err(Error::IdentifierNotReferenced(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Pop the referenced identifiers and copy them to the specified
    /// `ContentWriter`.
    ///
    /// # Errors
    ///
    /// If the copy fails after some elements have been copied,
    /// `Error::CopyInterrupted` is returned which contains the list of
    /// identifiers that have not been copied.
    async fn pop_referenced_identifiers_and_copy_to(
        &self,
        content_writer: impl ContentWriter + Send + Sync + 'async_trait,
    ) -> Result<()> {
        let identifiers = self.pop_referenced_identifiers().await?;

        self.copy_to(content_writer, identifiers).await
    }
}

impl<T: ContentTracker + ContentReaderExt> ContentTrackerExt for T {}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressReader: Display {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::IdentifierNotFound` is
    /// returned.
    async fn get_content_read_address_with_origin(
        &self,
        id: &Identifier,
    ) -> Result<(String, Origin)>;
}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressWriter: Display {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier already exists, `Error::IdentifierAlreadyExists` is returned.
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String>;
}

/// `ContentAddressProvider` is trait for all types that are both address readers and writers.
pub trait ContentAddressProvider: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementation of `ContentAddressProvider`.
impl<T> ContentAddressProvider for T where T: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementations for Arc<T> variants.

#[async_trait]
impl<T: ContentReader + Send + Sync> ContentReader for Arc<T> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        self.as_ref().get_content_readers(ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.as_ref().resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ContentWriter for Arc<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.as_ref().register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentTracker + Send + Sync> ContentTracker for Arc<T> {
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        self.as_ref().remove_content(id).await
    }

    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        self.as_ref().pop_referenced_identifiers().await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync> ContentAddressReader for Arc<T> {
    async fn get_content_read_address_with_origin(
        &self,
        id: &Identifier,
    ) -> Result<(String, Origin)> {
        self.as_ref().get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync> ContentAddressWriter for Arc<T> {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for Box<T> variants.

#[async_trait]
impl<T: ContentReader + Send + Sync + ?Sized> ContentReader for Box<T> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        self.as_ref().get_content_readers(ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.as_ref().resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for Box<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.as_ref().register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentTracker + Send + Sync + ?Sized> ContentTracker for Box<T> {
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        self.as_ref().remove_content(id).await
    }

    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        self.as_ref().pop_referenced_identifiers().await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for Box<T> {
    async fn get_content_read_address_with_origin(
        &self,
        id: &Identifier,
    ) -> Result<(String, Origin)> {
        self.as_ref().get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync + ?Sized> ContentAddressWriter for Box<T> {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for &T variants.

#[async_trait]
impl<T: ContentReader + Send + Sync + ?Sized> ContentReader for &T {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        (**self).get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        (**self).get_content_readers(ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        (**self).resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for &T {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        (**self).get_content_writer(id).await
    }
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        (**self).register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentTracker + Send + Sync + ?Sized> ContentTracker for &T {
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        (**self).remove_content(id).await
    }

    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        (**self).pop_referenced_identifiers().await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for &T {
    async fn get_content_read_address_with_origin(
        &self,
        id: &Identifier,
    ) -> Result<(String, Origin)> {
        (**self).get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync + ?Sized> ContentAddressWriter for &T {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_write_address(id).await
    }
}
