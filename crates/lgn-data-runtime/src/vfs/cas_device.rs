use async_trait::async_trait;
use lgn_content_store::ContentStore;

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    manifest: Manifest,
    content_store: Box<dyn ContentStore>,
}

impl CasDevice {
    pub(crate) fn new(manifest: Manifest, content_store: Box<dyn ContentStore>) -> Self {
        Self {
            manifest,
            content_store,
        }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        let (checksum, size) = self.manifest.find(type_id)?;
        let content = self.content_store.read(checksum).await?;
        assert_eq!(
            content.len(),
            size,
            "content size mismatch for asset {}, checksum: {}, expected size: {}, actual: {}",
            type_id,
            checksum,
            size,
            content.len()
        );
        Some(content)
    }

    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
