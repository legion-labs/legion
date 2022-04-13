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
    ContentTracker, ContentWriter, Error, Identifier, Origin, Result, Uploader, UploaderImpl,
};

type RefCountedData = (usize, Vec<u8>);

/// A `MemoryProvider` is a provider that stores content in RAM.
#[derive(Default, Debug, Clone)]
pub struct MemoryProvider {
    content_map: Arc<RwLock<HashMap<Identifier, RefCountedData>>>,
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
            Some((_, content)) => {
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
                            Some((_, content)) => Ok(std::io::Cursor::new(content.clone())
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

        if let Some((refcount, _)) = self.content_map.write().await.get_mut(id) {
            *refcount += 1;
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

#[async_trait]
impl ContentTracker for MemoryProvider {
    /// Decrease the reference count of the specified content.
    ///
    /// If the reference count is already zero, the call is a no-op and no error
    /// is returned.
    ///
    /// # Errors
    ///
    /// If the content was not referenced, `Error::IdentifierNotReferenced` is
    /// returned.
    ///
    /// This can happen when the content was existing prior to the tracker being
    /// instanciated, which is a very common case. This is not an error and
    /// callers should be prepared to handle this.
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        async_span_scope!("MemoryProvider::remove_content");

        if let Some((refcount, _)) = self.content_map.write().await.get_mut(id) {
            if *refcount > 0 {
                *refcount -= 1;
            }
        } else {
            return Err(Error::IdentifierNotReferenced(id.clone()));
        }

        Ok(())
    }

    /// Gets all the identifiers with a stricly positive reference count.
    ///
    /// The reference counts of the returned identifiers are guaranteed to be
    /// zero after the call.
    ///
    /// The typical use-case for this method is to synchronize all the returned
    /// identifiers to a more persistent content-store instance.
    ///
    /// This can be achieved more simply by calling the
    /// `pop_referenced_identifiers_and_copy_to` method from the
    /// `ContentTrackerExt` trait.
    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        async_span_scope!("MemoryProvider::pop_referenced_identifiers");

        let mut map = self.content_map.write().await;

        let mut ids = Vec::with_capacity(map.len());

        for (id, (refcount, _)) in map.iter_mut() {
            if *refcount > 0 {
                *refcount = 0;
                ids.push(id.clone());
            }
        }

        Ok(ids)
    }
}

type MemoryUploader = Uploader<MemoryUploaderImpl>;

#[derive(Debug)]
struct MemoryUploaderImpl {
    map: Arc<RwLock<HashMap<Identifier, RefCountedData>>>,
}

#[async_trait]
impl UploaderImpl for MemoryUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("MemoryProvider::upload");

        let mut map = self.map.write().await;

        // Let's make sure we handle the case where a concurrent write created the value before us.
        //
        // In that case we must increment the refcount properly.
        if let Some((refcount, content)) = map.get_mut(&id) {
            if content != &data {
                return Err(Error::Corrupt(id));
            }

            *refcount += 1;
        } else {
            map.insert(id, (1, data));
        }

        Ok(())
    }
}
