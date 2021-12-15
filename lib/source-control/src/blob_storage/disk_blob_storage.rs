use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use super::{BlobStorage, BoxedAsyncRead, BoxedAsyncWrite};

pub struct DiskBlobStorage(PathBuf);

impl DiskBlobStorage {
    pub async fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)
            .await
            .with_context(|| {
                format!(
                    "could not create local blobs directory at: {}",
                    root.display()
                )
            })
            .map(|_| Self(root))
    }
}

#[async_trait]
impl BlobStorage for DiskBlobStorage {
    async fn get_blob_reader(&self, hash: &str) -> super::Result<BoxedAsyncRead> {
        let blob_path = self.0.join(hash);

        let file = match fs::File::open(&blob_path).await {
            Ok(file) => file,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Err(super::Error::NoSuchBlob(hash.to_string()));
                }

                return Err(super::Error::forward_with_context(
                    err,
                    format!("could not open blob file: {}", blob_path.display()),
                ));
            }
        };

        Ok(Box::pin(file))
    }

    async fn get_blob_writer(&self, hash: &str) -> super::Result<Option<BoxedAsyncWrite>> {
        let blob_path = self.0.join(hash);

        if fs::metadata(&blob_path).await.is_ok() {
            return Ok(None);
        }

        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&blob_path)
            .await
            .map_err(|e| {
                super::Error::forward_with_context(
                    e,
                    format!("could not create blob file: {}", blob_path.display()),
                )
            })?;

        Ok(Some(Box::pin(file)))
    }
}
