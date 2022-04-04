use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store2::{ContentProvider, ContentReaderExt};

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    manifest: Manifest,
    content_store: Arc<Box<dyn ContentProvider + Send + Sync>>,
}

impl CasDevice {
    pub(crate) fn new(
        manifest: Manifest,
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
        let checksum = self.manifest.find(type_id)?;
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
    }

    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
