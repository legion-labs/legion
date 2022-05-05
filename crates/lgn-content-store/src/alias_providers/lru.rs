use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, span_scope};
use lru::LruCache;
use tokio::sync::Mutex;

use super::{Alias, AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

/// A `LruAliasProvider` is a provider that stores content in RAM, but only keeps a certain amount of content, by evicting older, less recently accessed, data.
#[derive(Debug, Clone)]
pub struct LruAliasProvider {
    alias_map: Arc<Mutex<LruCache<Alias, Identifier>>>,
    size: usize,
}

impl LruAliasProvider {
    /// Creates a new `LruAliasProvider` instance who stores content in the
    /// process memory.
    pub fn new(size: usize) -> Self {
        span_scope!("LruAliasProvider::new");

        debug!("LruAliasProvider::new(size: {})", size);

        Self {
            alias_map: Arc::new(Mutex::new(LruCache::new(size))),
            size,
        }
    }
}

impl Display for LruAliasProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lru (size: {})", self.size)
    }
}

#[async_trait]
impl AliasReader for LruAliasProvider {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("LruAliasProvider::resolve_alias");

        let mut map = self.alias_map.lock().await;
        let key = key.into();

        map.get(&key).cloned().ok_or(Error::AliasNotFound(key))
    }
}

#[async_trait]
impl AliasWriter for LruAliasProvider {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("LruAliasProvider::register_alias");

        let key = key.into();
        let mut map = self.alias_map.lock().await;

        if map.contains(&key) {
            return Err(Error::AliasAlreadyExists(key));
        }

        map.put(key.clone(), id.clone());

        Ok(Identifier::new_alias(key))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_lru_alias_provider() {
        let alias_provider = LruAliasProvider::new(2);

        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;
    }
}
