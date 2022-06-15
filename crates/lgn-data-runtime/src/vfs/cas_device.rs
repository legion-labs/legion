use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{ResourceIndex, ResourceReader, SharedTreeIdentifier},
    Provider,
};

use super::Device;
use crate::{new_resource_type_and_id_indexer, ResourceTypeAndId, ResourceTypeAndIdIndexer};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    provider: Arc<Provider>,
    manifest: ResourceIndex<ResourceTypeAndIdIndexer>,
}

impl CasDevice {
    pub(crate) fn new(provider: Arc<Provider>, manifest_id: SharedTreeIdentifier) -> Self {
        let manifest = ResourceIndex::new_shared_with_id(
            Arc::clone(&provider),
            new_resource_type_and_id_indexer(),
            manifest_id,
        );
        Self { provider, manifest }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(resource_id)) = self.manifest.get_identifier(&type_id.into()).await {
            if let Ok(resource_bytes) = self.provider.read_resource_as_bytes(&resource_id).await {
                return Some(resource_bytes);
            }
        }

        None
    }

    async fn reload(&mut self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
