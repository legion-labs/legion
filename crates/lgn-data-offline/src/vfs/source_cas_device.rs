use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{ResourceIndex, ResourceReader, SharedTreeIdentifier},
    Provider,
};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, Device, ResourceTypeAndId, ResourceTypeAndIdIndexer,
};

use crate::resource::deserialize_and_skip_metadata;

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct SourceCasDevice {
    persistent_provider: Arc<Provider>,
    volatile_provider: Arc<Provider>,
    manifest: ResourceIndex<ResourceTypeAndIdIndexer>,
}

impl SourceCasDevice {
    pub(crate) fn new(
        persistent_provider: Arc<Provider>,
        volatile_provider: Arc<Provider>,
        manifest_id: SharedTreeIdentifier,
    ) -> Self {
        Self {
            persistent_provider,
            volatile_provider,
            manifest: ResourceIndex::new_shared_with_id(
                new_resource_type_and_id_indexer(),
                manifest_id,
            ),
        }
    }
}

#[async_trait]
impl Device for SourceCasDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(resource_id)) = self
            .manifest
            .get_identifier(&self.volatile_provider, &type_id.into())
            .await
        {
            if let Ok(resource_bytes) = self
                .persistent_provider
                .read_resource_as_bytes(&resource_id)
                .await
            {
                let mut reader = std::io::Cursor::new(resource_bytes);

                // skip over the pre-pended metadata
                deserialize_and_skip_metadata(&mut reader);

                let pos = reader.position() as usize;
                let resource_bytes = reader.into_inner();

                return Some(resource_bytes[pos..].to_vec());
            }
        }

        None
    }

    async fn reload(&mut self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
