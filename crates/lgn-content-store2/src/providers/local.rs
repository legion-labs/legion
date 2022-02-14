use anyhow::Context;
use async_trait::async_trait;
use std::{path::PathBuf, pin::Pin};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ContentReader, ContentWriter, Error, Identifier, Result};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
pub struct LocalProvider(PathBuf);

impl LocalProvider {
    /// Creates a new `LocalProvider` instance who stores content in the
    /// specified directory.
    ///
    /// # Errors
    ///
    /// If the directory does not exist, or it cannot be created, an error is
    /// returned.
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        tokio::fs::create_dir_all(&root)
            .await
            .with_context(|| format!("could not create local provider in: {}", root.display()))
            .map(|_| Self(root))
            .map_err(Into::into)
    }
}

#[async_trait]
impl ContentReader for LocalProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        let path = self.0.join(id.to_string());

        match tokio::fs::File::open(&path).await {
            Ok(file) => match file.metadata().await {
                Ok(metadata) => {
                    if metadata.len() != id.data_size() {
                        Err(Error::Corrupt {})
                    } else {
                        Ok(Box::pin(file))
                    }
                }
                Err(err) => Err(anyhow::anyhow!(
                    "could not get metadata for file at `{}`: {}",
                    path.display(),
                    err
                )
                .into()),
            },
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    Err(Error::NotFound {})
                } else {
                    Err(
                        anyhow::anyhow!("could not open file at `{}`: {}", path.display(), e)
                            .into(),
                    )
                }
            }
        }
    }
}

#[async_trait]
impl ContentWriter for LocalProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        let path = self.0.join(id.to_string());

        if tokio::fs::metadata(&path).await.is_ok() {
            return Err(Error::AlreadyExists {});
        }

        match tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .await
        {
            Ok(file) => Ok(Box::pin(file)),
            Err(err) => {
                Err(anyhow::anyhow!("could not open file at `{}`: {}", path.display(), err).into())
            }
        }
    }
}
