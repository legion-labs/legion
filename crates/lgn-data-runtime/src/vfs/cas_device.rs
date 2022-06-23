use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{ResourceIndex, ResourceReader, SharedTreeIdentifier},
    Provider,
};

use super::Device;
use crate::ResourceTypeAndId;

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    volatile_provider: Arc<Provider>,
    runtime_manifest: ResourceIndex<ResourceTypeAndId>,
}

impl CasDevice {
    pub(crate) fn new(
        volatile_provider: Arc<Provider>,
        runtime_manifest_id: SharedTreeIdentifier,
    ) -> Self {
        let runtime_manifest =
            ResourceIndex::new_shared_with_id(Arc::clone(&volatile_provider), runtime_manifest_id);
        Self {
            volatile_provider,
            runtime_manifest,
        }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(resource_id)) = self.runtime_manifest.get_identifier(type_id).await {
            if let Ok(resource_bytes) = self
                .volatile_provider
                .read_resource_as_bytes(&resource_id)
                .await
            {
                return Some(resource_bytes);
            }
        }

        None
    }

    async fn reload(&mut self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
