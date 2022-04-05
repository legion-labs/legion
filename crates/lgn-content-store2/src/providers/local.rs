use anyhow::Context;
use async_trait::async_trait;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::PathBuf,
};

use crate::{
    traits::get_content_readers_impl, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result,
};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
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

    fn mangled_file_path(key: &str) -> Result<String> {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);
        enc.write_all(key.as_bytes())?;
        Ok(enc.into_inner())
    }
}

#[async_trait]
impl ContentReader for LocalProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let path = self.0.join(id.to_string());

        match tokio::fs::File::open(&path).await {
            Ok(file) => match file.metadata().await {
                Ok(metadata) => {
                    let metadata_size: usize = metadata
                        .len()
                        .try_into()
                        .expect("metadata size does not fit in usize"); // Should never happen on a modern architecture.

                    if metadata_size != id.data_size() {
                        Err(Error::Corrupt)
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
                    Err(Error::NotFound)
                } else {
                    Err(
                        anyhow::anyhow!("could not open file at `{}`: {}", path.display(), e)
                            .into(),
                    )
                }
            }
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        let alias_path = self
            .0
            .clone()
            .join(key_space)
            .join(Self::mangled_file_path(key)?);

        match tokio::fs::read_to_string(&alias_path).await {
            Ok(s) => s.parse(),
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    Err(Error::NotFound)
                } else {
                    Err(
                        anyhow::anyhow!("could not open file at `{}`: {}", alias_path.display(), e)
                            .into(),
                    )
                }
            }
        }
    }
}

#[async_trait]
impl ContentWriter for LocalProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let path = self.0.join(id.to_string());

        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            let metadata_size: usize = metadata
                .len()
                .try_into()
                .expect("metadata size does not fit in usize"); // Should never happen on a modern architecture.

            if id.data_size() == metadata_size {
                return Err(Error::AlreadyExists);
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

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        let mut alias_path = self.0.clone();
        alias_path.push(key_space);

        tokio::fs::create_dir_all(&alias_path).await?;

        alias_path.push(Self::mangled_file_path(key)?);

        tokio::fs::write(alias_path, format!("{}", id)).await?;
        Ok(())
    }
}
