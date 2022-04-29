use std::fmt::Display;

use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, span_scope, warn};

use super::{AliasProvider, AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

/// A `AliasProviderCache` is a provider that stores locally content that was retrieved from a remote source.
#[derive(Debug, Clone)]
pub struct AliasProviderCache<Remote, Local> {
    remote: Remote,
    local: Local,
}

impl<Remote: Display, Local: Display> AliasProviderCache<Remote, Local> {
    /// Creates a new `AliasProviderCache` instance who stores content in the
    /// backing remote and local providers.
    pub fn new(remote: Remote, local: Local) -> Self {
        span_scope!("AliasProviderCache::new");

        debug!(
            "AliasProviderCache::new(remote: {}, local: {})",
            remote, local
        );

        Self { remote, local }
    }
}

impl<Remote: Display, Local: Display> Display for AliasProviderCache<Remote, Local> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} cached by {})", self.remote, self.local)
    }
}

#[async_trait]
impl<Remote: AliasReader, Local: AliasProvider> AliasReader for AliasProviderCache<Remote, Local> {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("AliasProviderCache::resolve_alias");

        match self.local.resolve_alias(key).await {
            Ok(id) => Ok(id),
            Err(Error::AliasNotFound(_)) => match self.remote.resolve_alias(key).await {
                Ok(id) => {
                    if let Err(err) = self.local.register_alias(key, &id).await {
                        warn!(
                            "Failed to register alias {:02x?} in local cache: {}",
                            key, err
                        );
                    }

                    Ok(id)
                }
                Err(err) => Err(err),
            },
            // If the local provider fails, we just fall back to the remote without caching.
            Err(_) => self.remote.resolve_alias(key).await,
        }
    }
}

#[async_trait]
impl<Remote: AliasWriter, Local: AliasWriter> AliasWriter for AliasProviderCache<Remote, Local> {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("AliasProviderCache::register_alias");

        let result = self.remote.register_alias(key, id).await?;

        if let Err(err) = self.local.register_alias(key, id).await {
            warn!(
                "Failed to register alias {:02x?} in local cache: {}",
                key, err
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{LruAliasProvider, MemoryAliasProvider};

    use super::*;

    #[tokio::test]
    async fn test_cache_alias_provider() {
        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        let remote_alias_provider = Arc::new(MemoryAliasProvider::new());
        let local_alias_provider = Arc::new(LruAliasProvider::new(128));
        let alias_provider = AliasProviderCache::new(
            Arc::clone(&remote_alias_provider),
            Arc::clone(&local_alias_provider),
        );

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;
    }
}
