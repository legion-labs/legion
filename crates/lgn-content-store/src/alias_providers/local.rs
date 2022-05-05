use anyhow::Context;
use async_trait::async_trait;
use lgn_tracing::{async_span_scope, span_fn};
use std::{fmt::Display, io::Write, path::PathBuf};

use super::{AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

/// A `LocalAliasProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
pub struct LocalAliasProvider(PathBuf);

impl LocalAliasProvider {
    /// Creates a new `LocalAliasProvider` instance who stores content in the
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

    fn mangled_file_path(key: &[u8]) -> Result<String> {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);
        enc.write_all(key)?;
        Ok(enc.into_inner())
    }
}

impl Display for LocalAliasProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "local (root: {})", self.0.display())
    }
}

#[async_trait]
impl AliasReader for LocalAliasProvider {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("LocalAliasProvider::resolve_alias");

        let alias_path = self.0.clone().join(Self::mangled_file_path(key)?);

        match tokio::fs::read_to_string(&alias_path).await {
            Ok(s) => Ok(s.parse()?),
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    Err(Error::AliasNotFound(key.into()))
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
impl AliasWriter for LocalAliasProvider {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("LocalAliasProvider::register_alias");

        let mut alias_path = self.0.clone();

        tokio::fs::create_dir_all(&alias_path).await?;

        alias_path.push(Self::mangled_file_path(key)?);

        tokio::fs::write(alias_path, format!("{}", id)).await?;

        Ok(Identifier::new_alias(key.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_local_alias_provider() {
        let root = tempfile::tempdir().expect("failed to create temp directory");
        let alias_provider = LocalAliasProvider::new(root.path()).await.unwrap();

        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;
    }
}
