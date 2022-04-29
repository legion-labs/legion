use anyhow::Context;
use async_trait::async_trait;
use lgn_tracing::{async_span_scope, span_fn};
use std::{fmt::Display, path::PathBuf};

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Origin, Result, WithOriginAndSize,
};

/// A `LocalContentProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
pub struct LocalContentProvider(PathBuf);

impl LocalContentProvider {
    /// Creates a new `LocalContentProvider` instance who stores content in the
    /// specified directory.
    ///
    /// # Errors
    ///
    /// If the directory does not exist, or it cannot be created, an error is
    /// returned.
    #[span_fn]
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        tokio::fs::create_dir_all(&root)
            .await
            .with_context(|| format!("could not create local provider in: {}", root.display()))
            .map(|_| Self(root))
            .map_err(Into::into)
    }
}

impl Display for LocalContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "local (root: {})", self.0.display())
    }
}

#[async_trait]
impl ContentReader for LocalContentProvider {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("LocalContentProvider::get_content_reader");

        let path = self.0.join(id.to_string());

        match tokio::fs::File::open(&path).await {
            Ok(file) => match file.metadata().await {
                Ok(metadata) => {
                    let metadata_size: usize = metadata
                        .len()
                        .try_into()
                        .expect("metadata size does not fit in usize"); // Should never happen on a modern architecture.

                    if metadata_size != id.data_size() {
                        Err(Error::CorruptedHashRef(id.clone()))
                    } else {
                        Ok(file.with_origin_and_size(Origin::Local { path }, id.data_size()))
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
                    Err(Error::HashRefNotFound(id.clone()))
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
impl ContentWriter for LocalContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("LocalContentProvider::get_content_writer");

        let path = self.0.join(id.to_string());

        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            let metadata_size: usize = metadata
                .len()
                .try_into()
                .expect("metadata size does not fit in usize"); // Should never happen on a modern architecture.

            if id.data_size() == metadata_size {
                return Err(Error::HashRefAlreadyExists(id.clone()));
            }
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_local_content_provider() {
        let root = tempfile::tempdir().expect("failed to create temp directory");
        let content_provider = LocalContentProvider::new(root.path()).await.unwrap();

        let data: &[u8; 128] = &[0x41; 128];

        let origin = Origin::Local {
            path: root.path().join(HashRef::new_from_data(data).to_string()),
        };

        crate::content_providers::test_content_provider(&content_provider, data, origin).await;
    }
}
