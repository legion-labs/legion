use std::{collections::HashMap, fmt::Display, sync::Arc};

use async_trait::async_trait;
use lgn_tracing::async_span_scope;
use tokio::sync::RwLock;

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Origin, Result, Uploader, UploaderImpl, WithOriginAndSize,
};

type RefCountedData = (usize, Vec<u8>);

/// A `MemoryContentProvider` is a provider that stores content in RAM.
#[derive(Default, Debug, Clone)]
pub struct MemoryContentProvider {
    content_map: Arc<RwLock<HashMap<HashRef, RefCountedData>>>,
}

impl MemoryContentProvider {
    /// Creates a new `MemoryContentProvider` instance who stores content in the
    /// process memory.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for MemoryContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in-memory")
    }
}

#[async_trait]
impl ContentReader for MemoryContentProvider {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("MemoryContentProvider::get_content_reader");

        let map = self.content_map.read().await;

        match map.get(id) {
            Some((_, content)) => Ok(std::io::Cursor::new(content.clone())
                .with_origin_and_size(Origin::Memory {}, id.data_size())),
            None => Err(Error::HashRefNotFound(id.clone())),
        }
    }
}

#[async_trait]
impl ContentWriter for MemoryContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("MemoryContentProvider::get_content_writer");

        if let Some((refcount, _)) = self.content_map.write().await.get_mut(id) {
            *refcount += 1;
            Err(Error::HashRefAlreadyExists(id.clone()))
        } else {
            Ok(Box::pin(MemoryUploader::new(MemoryUploaderImpl {
                map: Arc::clone(&self.content_map),
            })))
        }
    }
}

type MemoryUploader = Uploader<MemoryUploaderImpl>;

#[derive(Debug)]
struct MemoryUploaderImpl {
    map: Arc<RwLock<HashMap<HashRef, RefCountedData>>>,
}

#[async_trait]
impl UploaderImpl for MemoryUploaderImpl {
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("MemoryContentProvider::upload");

        let id = HashRef::new_from_data(&data);

        let mut map = self.map.write().await;

        // Let's make sure we handle the case where a concurrent write created the value before us.
        //
        // In that case we must increment the refcount properly.
        if let Some((refcount, content)) = map.get_mut(&id) {
            if content != &data {
                return Err(Error::CorruptedHashRef(id));
            }

            *refcount += 1;
        } else {
            map.insert(id, (1, data));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_memory_content_provider() {
        let content_provider = MemoryContentProvider::new();

        let data: &[u8; 128] = &[0x41; 128];

        let origin = Origin::Memory {};

        crate::content_providers::test_content_provider(&content_provider, data, origin).await;
    }
}
