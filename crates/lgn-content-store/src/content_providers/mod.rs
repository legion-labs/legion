//! Content providers for various backends.

#[cfg(feature = "aws")]
mod aws_dynamodb;
#[cfg(feature = "aws")]
mod aws_s3;
mod cache;
mod errors;
mod grpc;
mod hash_ref;
mod local;
#[cfg(feature = "lru")]
mod lru;
mod memory;
mod monitor;
mod origin;
#[cfg(feature = "redis")]
mod redis;
mod uploader;

use std::{
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
};

#[cfg(feature = "lru")]
pub use self::lru::LruContentProvider;
#[cfg(feature = "redis")]
pub use self::redis::RedisContentProvider;
use async_trait::async_trait;
#[cfg(feature = "aws")]
pub use aws_dynamodb::AwsDynamoDbContentProvider;
#[cfg(feature = "aws")]
pub use aws_s3::{AwsS3ContentProvider, AwsS3Url};
pub use cache::ContentProviderCache;
pub use errors::{Error, Result};
pub use grpc::GrpcContentProvider;
pub use hash_ref::{HashAlgorithm, HashRef, InvalidHashRef};
use lgn_tracing::async_span_scope;
pub use local::LocalContentProvider;
pub use memory::MemoryContentProvider;
pub use monitor::{ContentProviderMonitor, MonitorAsyncAdapter, TransferCallbacks};
pub use origin::Origin;

use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub(crate) use uploader::{Uploader, UploaderImpl};

/// A trait that extends `AsyncRead` with an origin.
pub trait AsyncReadWithOriginAndSize: AsyncRead {
    fn size(&self) -> usize;
    fn origin(&self) -> &Origin;
}

impl<T: AsyncReadWithOriginAndSize> AsyncReadWithOriginAndSize for Pin<Box<T>> {
    fn size(&self) -> usize {
        (**self).size()
    }

    fn origin(&self) -> &Origin {
        (**self).origin()
    }
}

#[pin_project]
struct AsyncReadWithOriginAndSizeImpl<R: AsyncRead> {
    #[pin]
    inner: R,
    size: usize,
    origin: Origin,
}

impl<R: AsyncRead> AsyncReadWithOriginAndSizeImpl<R> {
    pub(crate) fn new(inner: R, origin: Origin, size: usize) -> Self {
        Self {
            inner,
            size,
            origin,
        }
    }
}

impl<R: AsyncRead> AsyncRead for AsyncReadWithOriginAndSizeImpl<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<R: AsyncRead> AsyncReadWithOriginAndSize for AsyncReadWithOriginAndSizeImpl<R> {
    fn size(&self) -> usize {
        self.size
    }

    fn origin(&self) -> &Origin {
        &self.origin
    }
}

pub(crate) trait WithOriginAndSize {
    fn with_origin_and_size(self, origin: Origin, size: usize)
        -> ContentAsyncReadWithOriginAndSize;
}

impl<R: AsyncRead + Send + 'static> WithOriginAndSize for R {
    fn with_origin_and_size(
        self,
        origin: Origin,
        size: usize,
    ) -> ContentAsyncReadWithOriginAndSize {
        Box::pin(AsyncReadWithOriginAndSizeImpl::new(self, origin, size))
    }
}

/// A reader as returned by the `ContentReader` trait.
pub type ContentAsyncReadWithOriginAndSize = Pin<Box<dyn AsyncReadWithOriginAndSize + Send>>;

/// A reader as returned by a chunker.
pub type ContentAsyncRead = Pin<Box<dyn AsyncRead + Send>>;

/// A writer as returned by the `ContentWriter` trait.
pub type ContentAsyncWrite = Pin<Box<dyn AsyncWrite + Send>>;

/// ContentReader is a trait for reading content from a content-store.
#[async_trait]
pub trait ContentReader: Display + Debug + Send + Sync {
    /// Returns an async reader that reads the content referenced by the
    /// specified identifier.
    ///
    /// If the identifier does not match any content, `Error::HashRefNotFound` is
    /// returned.
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize>;
}

#[async_trait]
pub trait ContentReaderExt: ContentReader {
    /// Check whether the identifier exists in the store.
    async fn exists(&self, id: &HashRef) -> Result<bool> {
        async_span_scope!("ContentReaderExt::exists");

        match self.get_content_reader(id).await {
            Ok(_) => Ok(true),
            Err(Error::HashRefNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Read the origin for a given identifier.
    async fn read_origin(&self, id: &HashRef) -> Result<Origin> {
        async_span_scope!("ContentReaderExt::read_origin");

        self.get_content_reader(id)
            .await
            .map(|r| r.origin().clone())
    }

    /// Read the content referenced by the specified identifier with its origin.
    async fn read_content_with_origin(&self, id: &HashRef) -> Result<(Vec<u8>, Origin)> {
        async_span_scope!("ContentReaderExt::read_content_with_origin");

        let mut reader = self.get_content_reader(id).await?;

        let mut result = Vec::with_capacity(id.data_size());

        reader
            .read_to_end(&mut result)
            .await
            .map_err(Into::into)
            .map(|_| (result, reader.origin().clone()))
    }

    /// Read the content referenced by the specified identifier.
    async fn read_content(&self, id: &HashRef) -> Result<Vec<u8>> {
        async_span_scope!("ContentReaderExt::read_content");

        self.read_content_with_origin(id)
            .await
            .map(|(data, _)| data)
    }
}

impl<T: ContentReader> ContentReaderExt for T {}

/// ContentWriter is a trait for writing content to a content-store.
#[async_trait]
pub trait ContentWriter: Display + Debug + Send + Sync {
    /// Returns an async write to which the content referenced by the specified
    /// specified identifier can be written.
    ///
    /// # Note
    ///
    /// The caller is responsible for writing exactly the data that matches with
    /// the identifier. Failure to do so could have catastrophic consequences
    /// and pollute the whole content-store space.
    ///
    /// This call in only intended for low-level efficient operations and very
    /// error prone. If possible, use the `write_content` method instead.
    ///
    /// # Errors
    ///
    /// If the data already exists, `Error::HashRefAlreadyExists` is returned and the
    /// caller should consider that the write operation is not necessary.
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite>;
}

#[async_trait]
pub trait ContentWriterExt: ContentWriter {
    /// Write the specified content and returns the newly associated identifier.
    ///
    /// If the content already exists, the write is a no-op and no error is
    /// returned.
    async fn write_content(&self, data: &[u8]) -> Result<HashRef> {
        async_span_scope!("ContentReaderExt::write_content");

        let id = HashRef::new_from_data(data);

        let mut writer = match self.get_content_writer(&id).await {
            Ok(writer) => writer,
            Err(Error::HashRefAlreadyExists(_)) => return Ok(id),
            Err(err) => return Err(err),
        };

        writer.write_all(data).await?;
        writer.shutdown().await?;

        Ok(id)
    }
}

impl<T: ContentWriter> ContentWriterExt for T {}

/// `ContentProvider` is trait for all types that are both readers and writers.
pub trait ContentProvider: ContentReader + ContentWriter {}

/// Blanket implementation of `ContentProvider`.
impl<T> ContentProvider for T where T: ContentReader + ContentWriter {}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressReader: Display + Debug + Send + Sync {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::HashRefNotFound` is
    /// returned.
    async fn get_content_read_address_with_origin(&self, id: &HashRef) -> Result<(String, Origin)>;
}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressWriter: Display + Debug + Send + Sync {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier already exists, `Error::HashRefAlreadyExists` is returned.
    async fn get_content_write_address(&self, id: &HashRef) -> Result<String>;
}

/// `ContentAddressProvider` is trait for all types that are both address readers and writers.
pub trait ContentAddressProvider: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementation of `ContentAddressProvider`.
impl<T> ContentAddressProvider for T where T: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementations for Arc<T> variants.

#[async_trait]
impl<T: ContentReader> ContentReader for Arc<T> {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        self.as_ref().get_content_reader(id).await
    }
}

#[async_trait]
impl<T: ContentWriter> ContentWriter for Arc<T> {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader> ContentAddressReader for Arc<T> {
    async fn get_content_read_address_with_origin(&self, id: &HashRef) -> Result<(String, Origin)> {
        self.as_ref().get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter> ContentAddressWriter for Arc<T> {
    async fn get_content_write_address(&self, id: &HashRef) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for Box<T> variants.

#[async_trait]
impl<T: ContentReader + ?Sized> ContentReader for Box<T> {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        self.as_ref().get_content_reader(id).await
    }
}

#[async_trait]
impl<T: ContentWriter + ?Sized> ContentWriter for Box<T> {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + ?Sized> ContentAddressReader for Box<T> {
    async fn get_content_read_address_with_origin(&self, id: &HashRef) -> Result<(String, Origin)> {
        self.as_ref().get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + ?Sized> ContentAddressWriter for Box<T> {
    async fn get_content_write_address(&self, id: &HashRef) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for &T variants.

#[async_trait]
impl<T: ContentReader + ?Sized> ContentReader for &T {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        (**self).get_content_reader(id).await
    }
}

#[async_trait]
impl<T: ContentWriter + ?Sized> ContentWriter for &T {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        (**self).get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + ?Sized> ContentAddressReader for &T {
    async fn get_content_read_address_with_origin(&self, id: &HashRef) -> Result<(String, Origin)> {
        (**self).get_content_read_address_with_origin(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + ?Sized> ContentAddressWriter for &T {
    async fn get_content_write_address(&self, id: &HashRef) -> Result<String> {
        (**self).get_content_write_address(id).await
    }
}

#[cfg(test)]
async fn test_content_provider(
    content_provider: impl ContentProvider,
    data: &[u8],
    origin: crate::Origin,
) -> crate::HashRef {
    let id = crate::HashRef::new_from_data(data);

    match content_provider.get_content_reader(&id).await {
        Ok(_) => panic!("expected HashRefNotFound error"),
        Err(Error::HashRefNotFound(err_id)) => assert_eq!(err_id, id),
        Err(err) => panic!("unexpected error: {}", err),
    };

    let new_id = crate::ContentWriterExt::write_content(&content_provider, data)
        .await
        .unwrap();
    assert_eq!(new_id, id);

    let (new_data, new_origin) =
        crate::ContentReaderExt::read_content_with_origin(&content_provider, &id)
            .await
            .unwrap();

    assert_eq!(new_data, data);
    assert_eq!(new_origin, origin);

    // Another write should yield `Error::HashRefAlreadyExists`.

    match content_provider.get_content_writer(&id).await {
        Ok(_) => panic!("expected HashRefAlreadyExists error"),
        Err(Error::HashRefAlreadyExists(err_id)) => assert_eq!(err_id, id),
        Err(err) => panic!("unexpected error: {}", err),
    };

    id
}
