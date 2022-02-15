use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, Identifier, Result};

/// ContentReader is a trait for reading content from a content-store.
#[async_trait]
pub trait ContentReader {
    /// Returns an async reader that reads the content referenced by the
    /// specified identifier.
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>>;

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
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>>;

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
