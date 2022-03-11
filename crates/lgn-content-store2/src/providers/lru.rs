use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use lru::LruCache;
use tokio::sync::Mutex;

use crate::{
    AliasRegisterer, AliasResolver, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result, Uploader, UploaderImpl,
};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
pub struct LruProvider {
    content_map: Arc<Mutex<LruCache<Identifier, Vec<u8>>>>,
    alias_map: Arc<Mutex<LruCache<(String, String), Identifier>>>,
}

impl LruProvider {
    /// Creates a new `LruProvider` instance who stores content in the
    /// process memory.
    pub fn new(size: usize) -> Self {
        Self {
            content_map: Arc::new(Mutex::new(LruCache::new(size))),
            alias_map: Arc::new(Mutex::new(LruCache::new(size))),
        }
    }
}

#[async_trait]
impl AliasResolver for LruProvider {
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        let mut map = self.alias_map.lock().await;
        let k = (key_space.to_string(), key.to_string());

        map.get(&k).cloned().ok_or(Error::NotFound)
    }
}

#[async_trait]
impl AliasRegisterer for LruProvider {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        let k = (key_space.to_string(), key.to_string());
        let mut map = self.alias_map.lock().await;

        if map.contains(&k) {
            return Err(Error::AlreadyExists);
        }

        map.put(k, id.clone());

        Ok(())
    }
}

#[async_trait]
impl ContentReader for LruProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let mut map = self.content_map.lock().await;

        match map.get(id) {
            Some(content) => Ok(Box::pin(std::io::Cursor::new(content.clone()))),
            None => Err(Error::NotFound),
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        let mut map = self.content_map.lock().await;

        let res = ids
            .iter()
            .map(|id| {
                (
                    id,
                    match map.get(id) {
                        Some(content) => {
                            Ok(Box::pin(std::io::Cursor::new(content.clone())) as ContentAsyncRead)
                        }
                        None => Err(Error::NotFound),
                    },
                )
            })
            .collect::<BTreeMap<_, Result<_>>>();

        Ok(res)
    }
}

#[async_trait]
impl ContentWriter for LruProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        if self.content_map.lock().await.get(id).is_some() {
            Err(Error::AlreadyExists)
        } else {
            Ok(Box::pin(MemoryUploader::new(
                id.clone(),
                MemoryUploaderImpl {
                    map: Arc::clone(&self.content_map),
                },
            )))
        }
    }
}

type MemoryUploader = Uploader<MemoryUploaderImpl>;

struct MemoryUploaderImpl {
    map: Arc<Mutex<LruCache<Identifier, Vec<u8>>>>,
}

#[async_trait]
impl UploaderImpl for MemoryUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        let mut map = self.map.lock().await;

        map.put(id, data);

        Ok(())
    }
}
