use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{ResourceIndex, ResourceReader, SharedTreeIdentifier},
    Provider,
};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, AssetRegistryReader, Device, ResourceTypeAndId,
    ResourceTypeAndIdIndexer,
};
//use crate::deserialize_and_skip_metadata;

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct SourceCasDevice {
    persistent_provider: Arc<Provider>,
    source_manifest: ResourceIndex<ResourceTypeAndIdIndexer>,
}

impl SourceCasDevice {
    pub(crate) fn new(
        persistent_provider: Arc<Provider>,
        source_manifest_id: SharedTreeIdentifier,
    ) -> Self {
        let source_manifest = ResourceIndex::new_shared_with_id(
            Arc::clone(&persistent_provider),
            new_resource_type_and_id_indexer(),
            source_manifest_id,
        );
        Self {
            persistent_provider,
            source_manifest,
        }
    }
}

#[async_trait]
impl Device for SourceCasDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(resource_id)) = self.source_manifest.get_identifier(&type_id.into()).await {
            if let Ok(resource_bytes) = self
                .persistent_provider
                .read_resource_as_bytes(&resource_id)
                .await
            {
                return Some(resource_bytes);
            }
        }
        None
    }

    async fn get_reader(&self, type_id: ResourceTypeAndId) -> Option<AssetRegistryReader> {
        if let Ok(Some(resource_id)) = self.source_manifest.get_identifier(&type_id.into()).await {
            if let Ok(reader) = self
                .persistent_provider
                .get_reader(resource_id.as_identifier())
                .await
            {
                return Some(Box::pin(reader) as AssetRegistryReader);
            }
        }
        None
    }

    async fn reload(&mut self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
