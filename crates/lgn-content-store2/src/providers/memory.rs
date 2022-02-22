use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    traits::get_content_readers_impl, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result, Uploader, UploaderImpl,
};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
#[derive(Default)]
pub struct MemoryProvider {
    map: Arc<RwLock<HashMap<Identifier, Vec<u8>>>>,
}

impl MemoryProvider {
    /// Creates a new `MemoryProvider` instance who stores content in the
    /// process memory.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ContentReader for MemoryProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let map = self.map.read().await;

        match map.get(id) {
            Some(content) => Ok(Box::pin(std::io::Cursor::new(content.clone()))),
            None => Err(Error::NotFound),
        }
    }

    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }
}

#[async_trait]
impl ContentWriter for MemoryProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        if self.map.read().await.contains_key(id) {
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
    map: Arc<RwLock<HashMap<Identifier, Vec<u8>>>>,
}

#[async_trait]
impl UploaderImpl for MemoryUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        let mut map = self.map.write().await;

        map.insert(id, data);

        Ok(())
    }
}
