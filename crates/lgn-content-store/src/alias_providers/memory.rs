use std::{collections::HashMap, fmt::Display, sync::Arc};

use async_trait::async_trait;
use lgn_tracing::async_span_scope;
use tokio::sync::RwLock;

use super::{Alias, AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

/// A `MemoryAliasProvider` is a provider that stores content in RAM.
#[derive(Default, Debug, Clone)]
pub struct MemoryAliasProvider {
    alias_map: Arc<RwLock<HashMap<Alias, Identifier>>>,
}

impl MemoryAliasProvider {
    /// Creates a new `MemoryAliasProvider` instance who stores content in the
    /// process memory.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for MemoryAliasProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in-memory")
    }
}

#[async_trait]
impl AliasReader for MemoryAliasProvider {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("MemoryAliasProvider::resolve_alias");

        let map = self.alias_map.read().await;
        let key = key.into();

        map.get(&key).cloned().ok_or(Error::AliasNotFound(key))
    }
}

#[async_trait]
impl AliasWriter for MemoryAliasProvider {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("MemoryAliasProvider::register_alias");

        let key = key.into();

        if self.alias_map.read().await.contains_key(&key) {
            return Err(Error::AliasAlreadyExists(key));
        }

        self.alias_map.write().await.insert(key.clone(), id.clone());

        Ok(Identifier::new_alias(key))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_memory_alias_provider() {
        let alias_provider = MemoryAliasProvider::new();

        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;
    }
}
