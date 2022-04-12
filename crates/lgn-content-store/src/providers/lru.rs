use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    sync::Arc,
};

use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, span_scope, warn};
use lru::LruCache;
use tokio::sync::Mutex;

use crate::{
    traits::WithOrigin, ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Origin, Result, Uploader, UploaderImpl,
};

/// A `LruProvider` is a provider that stores content in RAM, but only keeps a certain amount of content, by evicting older, less recently accessed, data.
#[derive(Debug, Clone)]
pub struct LruProvider {
    content_map: Arc<Mutex<LruCache<Identifier, Vec<u8>>>>,
    alias_map: Arc<Mutex<LruCache<(String, String), Identifier>>>,
    size: usize,
}

impl LruProvider {
    /// Creates a new `LruProvider` instance who stores content in the
    /// process memory.
    pub fn new(size: usize) -> Self {
        span_scope!("LruProvider::new");

        debug!("LruProvider::new(size: {})", size);

        Self {
            content_map: Arc::new(Mutex::new(LruCache::new(size))),
            alias_map: Arc::new(Mutex::new(LruCache::new(size))),
            size,
        }
    }
}

impl Display for LruProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lru (size: {})", self.size)
    }
}

#[async_trait]
impl ContentReader for LruProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("LruProvider::get_content_reader");

        let mut map = self.content_map.lock().await;

        match map.get(id) {
            Some(content) => {
                debug!("LruProvider::get_content_reader({}) -> item exists", id);

                Ok(std::io::Cursor::new(content.clone()).with_origin(Origin::Lru {}))
            }
            None => {
                warn!("LruProvider::get_content_reader({}) -> item not found", id);

                Err(Error::IdentifierNotFound(id.clone()))
            }
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        async_span_scope!("LruProvider::get_content_readers");

        debug!("LruProvider::get_content_readers(ids: {:?})", ids);

        let mut map = self.content_map.lock().await;

        let res = ids
            .iter()
            .map(|id| {
                (
                    id,
                    match map.get(id) {
                        Some(content) => {
                            debug!("LruProvider::get_content_readers({}) -> item exists", id);

                            Ok(std::io::Cursor::new(content.clone()).with_origin(Origin::Lru {}))
                        }
                        None => {
                            warn!("LruProvider::get_content_readers({}) -> item not found", id);

                            Err(Error::IdentifierNotFound(id.clone()))
                        }
                    },
                )
            })
            .collect::<BTreeMap<_, Result<_>>>();

        Ok(res)
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        async_span_scope!("LruProvider::resolve_alias");

        let mut map = self.alias_map.lock().await;
        let k = (key_space.to_string(), key.to_string());

        map.get(&k).cloned().ok_or_else(|| Error::AliasNotFound {
            key_space: key_space.to_string(),
            key: key.to_string(),
        })
    }
}

#[async_trait]
impl ContentWriter for LruProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("LruProvider::get_content_writer");

        debug!("LruProvider::get_content_writer({})", id);

        if self.content_map.lock().await.get(id).is_some() {
            debug!(
                "LruProvider::get_content_writer({}) -> content already present",
                id
            );

            Err(Error::IdentifierAlreadyExists(id.clone()))
        } else {
            debug!("LruProvider::get_content_writer({}) -> writer created", id);

            Ok(Box::pin(MemoryUploader::new(
                id.clone(),
                LruUploaderImpl {
                    map: Arc::clone(&self.content_map),
                },
            )))
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("LruProvider::register_alias");

        let k = (key_space.to_string(), key.to_string());
        let mut map = self.alias_map.lock().await;

        if map.contains(&k) {
            return Err(Error::AliasAlreadyExists {
                key_space: key_space.to_string(),
                key: key.to_string(),
            });
        }

        map.put(k, id.clone());

        Ok(())
    }
}

type MemoryUploader = Uploader<LruUploaderImpl>;

#[derive(Debug)]
struct LruUploaderImpl {
    map: Arc<Mutex<LruCache<Identifier, Vec<u8>>>>,
}

#[async_trait]
impl UploaderImpl for LruUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("LruProvider::upload");

        debug!("LruProvider::upload({})", id);

        let mut map = self.map.lock().await;

        map.put(id, data);

        Ok(())
    }
}
