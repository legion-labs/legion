use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{ChunkIdentifier, Chunker, ContentProvider, ContentReaderExt};
use lgn_tracing::error;

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    manifest: Option<Manifest>,
    content_store: Arc<Box<dyn ContentProvider + Send + Sync>>,
}

impl CasDevice {
    pub(crate) fn new(
        manifest: Option<Manifest>,
        content_store: Arc<Box<dyn ContentProvider + Send + Sync>>,
    ) -> Self {
        Self {
            manifest,
            content_store,
        }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Some(manifest) = &self.manifest {
            let checksum = manifest.find(type_id)?;
            let content = self.content_store.read_content(&checksum).await.ok()?;
            assert_eq!(
                content.len(),
                checksum.data_size(),
                "content size mismatch for asset {}, content_id: {}, expected size: {}, actual: {}",
                type_id,
                checksum,
                checksum.data_size(),
                content.len()
            );
            Some(content)
        } else {
            None
        }
    }

    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }

    async fn reload_manifest(&mut self, manifest_id: &ChunkIdentifier) {
        let chunker = Chunker::default();
        if let Ok(content) = chunker.read_chunk(&self.content_store, manifest_id).await {
            match serde_json::from_reader::<_, Manifest>(content.as_slice()) {
                Ok(manifest) => self.manifest = Some(manifest),
                Err(error) => error!("failed to read manifest contents: {}", error),
            }
        }
    }
}
