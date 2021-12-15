mod blob_storage_url;
mod disk_blob_storage;
mod error;
mod lz4;
mod s3_blob_storage;

pub use blob_storage_url::BlobStorageUrl;
pub use disk_blob_storage::DiskBlobStorage;
pub use error::{Error, Result};
pub use s3_blob_storage::{AwsS3Url, S3BlobStorage};

use std::{path::Path, pin::Pin};

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub type BoxedAsyncRead = Pin<Box<dyn AsyncRead + Send>>;
pub type BoxedAsyncWrite = Pin<Box<dyn AsyncWrite + Send>>;

#[async_trait]
pub trait BlobStorage: Send + Sync {
    /// Reads a blob from the storage.
    ///
    /// If no such blob exists, Error::NoSuchBlob is returned.
    ///
    /// In any other case, an error is returned.
    async fn get_blob_reader(&self, hash: &str) -> Result<BoxedAsyncRead>;

    /// Writes a blob to the storage.
    ///
    /// If the blob already exists, None is returned and no further action is
    /// required.
    ///
    /// In any other case, an error is returned.
    async fn get_blob_writer(&self, hash: &str) -> Result<Option<BoxedAsyncWrite>>;

    /// Reads the the full contents of a blob from the storage.
    async fn read_blob(&self, hash: &str) -> Result<Vec<u8>> {
        let mut reader = self.get_blob_reader(hash).await?;
        let mut contents = Vec::new();

        reader.read_to_end(&mut contents).await.map_err(|e| {
            Error::forward_with_context(e, format!("could not read blob: {}", hash))
        })?;

        Ok(contents)
    }

    /// Writes the full contents of a blob to the storage.
    async fn write_blob(&self, hash: &str, content: &[u8]) -> Result<()> {
        let writer = self.get_blob_writer(hash).await?;

        if let Some(mut writer) = writer {
            writer.write_all(content).await.map_err(|e| {
                Error::forward_with_context(e, format!("could not write blob: {}", hash))
            })?;
        }

        Ok(())
    }

    /// Download a blob from the storage and perist it to disk at the specified
    /// location.
    async fn download_blob(&self, path: &Path, hash: &str) -> Result<()> {
        let mut reader = self.get_blob_reader(hash).await?;
        let mut writer = tokio::fs::File::create(path).await.map_err(|e| {
            Error::forward_with_context(
                e,
                format!("could not create destination file: {}", path.display()),
            )
        })?;

        tokio::io::copy(&mut reader, &mut writer)
            .await
            .map_err(|e| {
                Error::forward_with_context(e, format!("could not copy blob data: {}", hash))
            })?;

        Ok(())
    }
}
