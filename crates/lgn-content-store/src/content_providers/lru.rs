use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, span_scope, warn};
use lru::LruCache;
use tokio::sync::Mutex;

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Origin, Result, Uploader, UploaderImpl, WithOriginAndSize,
};

/// A `LruContentProvider` is a provider that stores content in RAM, but only keeps a certain amount of content, by evicting older, less recently accessed, data.
#[derive(Debug, Clone)]
pub struct LruContentProvider {
    content_map: Arc<Mutex<LruCache<HashRef, Vec<u8>>>>,
    size: usize,
}

impl LruContentProvider {
    /// Creates a new `LruContentProvider` instance who stores content in the
    /// process memory.
    pub fn new(size: usize) -> Self {
        span_scope!("LruContentProvider::new");

        debug!("LruContentProvider::new(size: {})", size);

        Self {
            content_map: Arc::new(Mutex::new(LruCache::new(size))),
            size,
        }
    }
}

impl Display for LruContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lru (size: {})", self.size)
    }
}

#[async_trait]
impl ContentReader for LruContentProvider {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("LruContentProvider::get_content_reader");

        let mut map = self.content_map.lock().await;

        match map.get(id) {
            Some(content) => {
                debug!(
                    "LruContentProvider::get_content_reader({}) -> item exists",
                    id
                );

                Ok(std::io::Cursor::new(content.clone())
                    .with_origin_and_size(Origin::Lru {}, id.data_size()))
            }
            None => {
                warn!(
                    "LruContentProvider::get_content_reader({}) -> item not found",
                    id
                );

                Err(Error::HashRefNotFound(id.clone()))
            }
        }
    }
}

#[async_trait]
impl ContentWriter for LruContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("LruContentProvider::get_content_writer");

        debug!("LruContentProvider::get_content_writer({})", id);

        if self.content_map.lock().await.get(id).is_some() {
            debug!(
                "LruContentProvider::get_content_writer({}) -> content already present",
                id
            );

            Err(Error::HashRefAlreadyExists(id.clone()))
        } else {
            debug!(
                "LruContentProvider::get_content_writer({}) -> writer created",
                id
            );

            Ok(Box::pin(MemoryUploader::new(LruUploaderImpl {
                map: Arc::clone(&self.content_map),
            })))
        }
    }

    fn supports_unwrite(&self) -> bool {
        true
    }

    async fn unwrite_content(&self, id: &HashRef) -> Result<()> {
        async_span_scope!("LruContentProvider::unwrite");

        if self.content_map.lock().await.pop(id).is_some() {
            Ok(())
        } else {
            Err(Error::HashRefNotFound(id.clone()))
        }
    }
}

type MemoryUploader = Uploader<LruUploaderImpl>;

#[derive(Debug)]
struct LruUploaderImpl {
    map: Arc<Mutex<LruCache<HashRef, Vec<u8>>>>,
}

#[async_trait]
impl UploaderImpl for LruUploaderImpl {
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("LruContentProvider::upload");

        let id = HashRef::new_from_data(&data);

        debug!("LruContentProvider::upload({})", id);

        let mut map = self.map.lock().await;

        map.put(id, data);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{ContentReaderExt, ContentWriterExt};

    use super::*;

    #[tokio::test]
    async fn test_lru_content_provider() {
        let content_provider = LruContentProvider::new(2);

        let data: &[u8; 128] = &[0x41; 128];

        let origin = Origin::Lru {};

        let id =
            crate::content_providers::test_content_provider(&content_provider, data, origin).await;

        // Write enough content to make the LRU full.
        content_provider.write_content(&[0x42; 128]).await.unwrap();
        content_provider.write_content(&[0x43; 128]).await.unwrap();

        // The value should have been evicted.
        match content_provider.read_content(&id).await {
            Err(Error::HashRefNotFound(err_id)) => assert_eq!(err_id, id),
            Err(err) => panic!("Unexpected error: {}", err),
            Ok(..) => panic!("Expected error"),
        }

        // Rewrite the value and this time make frequent reads to avoid eviction.
        content_provider.write_content(data).await.unwrap();
        content_provider.read_content(&id).await.unwrap();
        content_provider.write_content(&[0x42; 128]).await.unwrap();
        content_provider.read_content(&id).await.unwrap();
        content_provider.write_content(&[0x43; 128]).await.unwrap();
        content_provider.read_content(&id).await.unwrap();
    }
}
