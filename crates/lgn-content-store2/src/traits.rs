use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
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

    /// Read the content referenced by the specified identifier.
    async fn read_content(&self, id: &Identifier) -> Result<Vec<u8>> {
        let mut reader = self.get_content_reader(id).await?;

        let mut result = Vec::new();

        reader
            .read_to_end(&mut result)
            .await
            .map_err(|err| anyhow::anyhow!("failed to read content: {}", err).into())
            .map(|_| result)
    }
}

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

    /// Write the specified content and returns the newly associated identifier.
    ///
    /// If the content already exists, the write is a no-op and no error is
    /// returned.
    async fn write_content(&self, data: &[u8]) -> Result<Identifier> {
        let id = Identifier::new_hash_ref_from_data(data);
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
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        self.as_ref().get_content_reader(id).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ContentWriter for Arc<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
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

/// Blanket implementations for &T variants.

#[async_trait]
impl<T: ContentReader + Send + Sync> ContentReader for &T {
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        (**self).get_content_reader(id).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ContentWriter for &T {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        (**self).get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync> ContentAddressReader for &T {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_read_address(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync> ContentAddressWriter for &T {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_write_address(id).await
    }
}
