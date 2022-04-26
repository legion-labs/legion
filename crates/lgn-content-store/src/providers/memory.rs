use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Display,
    sync::Arc,
};

use async_trait::async_trait;
use lgn_tracing::async_span_scope;
use tokio::sync::RwLock;

use crate::{
    traits::WithOrigin, ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Origin, Result, Uploader, UploaderImpl,
};

/// A `MemoryProvider` is a provider that stores content in RAM.
#[derive(Default, Debug, Clone)]
pub struct MemoryProvider {
    content_map: Arc<RwLock<HashMap<Identifier, Vec<u8>>>>,
    alias_map: Arc<RwLock<HashMap<(String, String), Identifier>>>,
}

impl MemoryProvider {
    /// Creates a new `MemoryProvider` instance who stores content in the
    /// process memory.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for MemoryProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in-memory")
    }
}

#[async_trait]
impl ContentReader for MemoryProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("MemoryProvider::get_content_reader");

        let map = self.content_map.read().await;

        match map.get(id) {
            Some(content) => {
                Ok(std::io::Cursor::new(content.clone()).with_origin(Origin::Memory {}))
            }
            None => Err(Error::IdentifierNotFound(id.clone())),
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        async_span_scope!("MemoryProvider::get_content_readers");

        let map = self.content_map.read().await;

        let res =
            ids.iter()
                .map(|id| {
                    (
                        id,
                        match map.get(id) {
                            Some(content) => Ok(std::io::Cursor::new(content.clone())
                                .with_origin(Origin::Memory {})),
                            None => Err(Error::IdentifierNotFound(id.clone())),
                        },
                    )
                })
                .collect::<BTreeMap<_, Result<_>>>();

        Ok(res)
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        async_span_scope!("MemoryProvider::resolve_alias");

        let map = self.alias_map.read().await;
        let k = (key_space.to_string(), key.to_string());

        map.get(&k).cloned().ok_or_else(|| Error::AliasNotFound {
            key_space: key_space.to_string(),
            key: key.to_string(),
        })
    }
}

#[async_trait]
impl ContentWriter for MemoryProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("MemoryProvider::get_content_writer");

        if self.content_map.read().await.contains_key(id) {
            Err(Error::IdentifierAlreadyExists(id.clone()))
        } else {
            Ok(Box::pin(MemoryUploader::new(
                id.clone(),
                MemoryUploaderImpl {
                    map: Arc::clone(&self.content_map),
                },
            )))
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("MemoryProvider::register_alias");

        let k = (key_space.to_string(), key.to_string());

        if self.alias_map.read().await.contains_key(&k) {
            return Err(Error::AliasAlreadyExists {
                key_space: key_space.to_string(),
                key: key.to_string(),
            });
        }

        self.alias_map.write().await.insert(k, id.clone());

        Ok(())
    }
}

type MemoryUploader = Uploader<MemoryUploaderImpl>;

#[derive(Debug)]
struct MemoryUploaderImpl {
    map: Arc<RwLock<HashMap<Identifier, Vec<u8>>>>,
}

#[async_trait]
impl UploaderImpl for MemoryUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("MemoryProvider::upload");

        let mut map = self.map.write().await;

        map.insert(id, data);

        Ok(())
    }
}
