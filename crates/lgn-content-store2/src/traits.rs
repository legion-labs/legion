use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use futures::future::join_all;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, Identifier, Result};

/// A reader as returned by the `ContentReader` trait.
pub type ContentAsyncRead = Pin<Box<dyn AsyncRead + Send>>;

/// A writer as returned by the `ContentWriter` trait.
pub type ContentAsyncWrite = Pin<Box<dyn AsyncWrite + Send>>;

/// ContentReader is a trait for reading content from a content-store.
#[async_trait]
pub trait ContentReader {
    /// Returns an async reader that reads the content referenced by the
    /// specified identifier.
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead>;

    /// Returns an async reader for each of the specified identifiers.
    ///
    /// If the content for a given identifier does not exist, `Error::NotFound`
    /// is returned instead.
    ///
    /// If the high-level request fails, an error is returned.
    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>>;
}

/// A default implementation for the `get_content_readers` method that just
/// calls in parallel `get_content_reader` for each identifier.
pub(crate) async fn get_content_readers_impl(
    reader: &(dyn ContentReader + Send + Sync),
    ids: &[Identifier],
) -> Result<Vec<Result<ContentAsyncRead>>> {
    let futures = ids
        .iter()
        .map(|id| reader.get_content_reader(id))
        .collect::<Vec<_>>();

    Ok(join_all(futures).await)
}

#[async_trait]
pub trait ContentReaderExt: ContentReader {
    /// Read the content referenced by the specified identifier.
    async fn read_content(&self, id: &Identifier) -> Result<Vec<u8>> {
        if let Identifier::Data(data) = id {
            return Ok(data.to_vec());
        }

        let mut reader = self.get_content_reader(id).await?;

        let mut result = Vec::new();

        reader
            .read_to_end(&mut result)
            .await
            .map_err(|err| anyhow::anyhow!("failed to read content: {}", err).into())
            .map(|_| result)
    }

    /// Read the contents referenced by the specified identifiers.
    async fn read_contents(&self, ids: &[Identifier]) -> Result<Vec<Result<Vec<u8>>>> {
        let readers = self.get_content_readers(ids).await?;
        let futures = readers
            .into_iter()
            .map(|r| async move {
                match r {
                    Ok(mut reader) => {
                        let mut result = Vec::new();
                        reader
                            .read_to_end(&mut result)
                            .await
                            .map_err(|err| {
                                anyhow::anyhow!("failed to read content: {}", err).into()
                            })
                            .map(|_| result)
                    }
                    Err(err) => Err(err),
                }
            })
            .collect::<Vec<_>>();

        Ok(join_all(futures).await)
    }
}

impl<T: ContentReader> ContentReaderExt for T {}

/// ContentWriter is a trait for writing content to a content-store.
#[async_trait]
pub trait ContentWriter {
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
    /// If the data already exists, `Error::AlreadyExists` is returned and the
    /// caller should consider that the write operation is not necessary.
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite>;
}

#[async_trait]
pub trait ContentWriterExt: ContentWriter {
    /// Write the specified content and returns the newly associated identifier.
    ///
    /// If the content already exists, the write is a no-op and no error is
    /// returned.
    async fn write_content(&self, data: &[u8]) -> Result<Identifier> {
        let id = Identifier::new(data);

        if id.is_data() {
            return Ok(id);
        }

        let mut writer = match self.get_content_writer(&id).await {
            Ok(writer) => writer,
            Err(Error::AlreadyExists) => return Ok(id),
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
}

impl<T: ContentWriter> ContentWriterExt for T {}

/// `ContentProvider` is trait for all types that are both readers and writers.
pub trait ContentProvider: ContentReader + ContentWriter {}

/// Blanket implementation of `ContentProvider`.
impl<T> ContentProvider for T where T: ContentReader + ContentWriter {}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressReader {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String>;
}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressWriter {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier already exists, `Error::AlreadyExists` is returned.
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String>;
}

/// `ContentAddressProvider` is trait for all types that are both address readers and writers.
pub trait ContentAddressProvider: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementation of `ContentAddressProvider`.
impl<T> ContentAddressProvider for T where T: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementations for Arc<T> variants.

#[async_trait]
impl<T: ContentReader + Send + Sync> ContentReader for Arc<T> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>> {
        self.as_ref().get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ContentWriter for Arc<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync> ContentAddressReader for Arc<T> {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_read_address(id).await
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
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>> {
        self.as_ref().get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for Box<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for Box<T> {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_read_address(id).await
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
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        (**self).get_content_reader(id).await
    }

    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>> {
        (**self).get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for &T {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        (**self).get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for &T {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_read_address(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync + ?Sized> ContentAddressWriter for &T {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_write_address(id).await
    }
}
