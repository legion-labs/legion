use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use lru::LruCache;
use tokio::sync::Mutex;

use crate::{
    ContentAsyncRead, ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier, Result,
    Uploader, UploaderImpl,
};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
pub struct LruProvider {
    map: Arc<Mutex<LruCache<Identifier, Vec<u8>>>>,
}

impl LruProvider {
    /// Creates a new `LruProvider` instance who stores content in the
    /// process memory.
    pub fn new(size: usize) -> Self {
        Self {
            map: Arc::new(Mutex::new(LruCache::new(size))),
        }
    }
}

#[async_trait]
impl ContentReader for LruProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let mut map = self.map.lock().await;

        match map.get(id) {
            Some(content) => Ok(Box::pin(std::io::Cursor::new(content.clone()))),
            None => Err(Error::NotFound),
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        let mut map = self.map.lock().await;

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
        if self.map.lock().await.get(id).is_some() {
            Err(Error::AlreadyExists)
        } else {
            Ok(Box::pin(MemoryUploader::new(
                id.clone(),
                MemoryUploaderImpl {
                    map: Arc::clone(&self.map),
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
