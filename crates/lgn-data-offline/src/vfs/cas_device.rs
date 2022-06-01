use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{BasicIndexer, ResourceByteReader, ResourceReader, TreeLeafNode},
    Provider,
};
use lgn_data_runtime::{
    manifest::ManifestId, new_resource_type_and_id_indexer, Device, ResourceTypeAndId,
    ResourceTypeAndIdIndexer,
};

use crate::resource::deserialize_and_skip_metadata;

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    provider: Arc<Provider>,
    indexer: ResourceTypeAndIdIndexer,
    manifest_id: Arc<ManifestId>,
}

impl CasDevice {
    pub(crate) fn new(provider: Arc<Provider>, manifest_id: Arc<ManifestId>) -> Self {
        Self {
            provider,
            indexer: new_resource_type_and_id_indexer(),
            manifest_id,
        }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(TreeLeafNode::Resource(leaf_id))) = self
            .indexer
            .get_leaf(&self.provider, &self.manifest_id.read(), &type_id.into())
            .await
        {
            if let Ok(resource_bytes) = self
                .provider
                .read_resource::<ResourceByteReader>(&leaf_id)
                .await
            {
                let mut reader = std::io::Cursor::new(resource_bytes.into_vec());

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
