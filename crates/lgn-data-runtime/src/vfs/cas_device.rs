use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{BasicIndexer, ResourceReader, SharedTreeIdentifier, TreeLeafNode},
    Provider,
};

use super::Device;
use crate::{new_resource_type_and_id_indexer, ResourceTypeAndId, ResourceTypeAndIdIndexer};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    provider: Arc<Provider>,
    indexer: ResourceTypeAndIdIndexer,
    manifest_id: SharedTreeIdentifier,
}

impl CasDevice {
    pub(crate) fn new(provider: Arc<Provider>, manifest_id: SharedTreeIdentifier) -> Self {
        Self {
            provider,
            indexer: new_resource_type_and_id_indexer(),
            manifest_id,
        }
    }

    pub(crate) async fn get_empty_manifest_id(provider: &Provider) -> ManifestId {
        ManifestId(provider.write_tree(&Tree::default()).await.unwrap())
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
            if let Ok(resource_bytes) = self.provider.read_resource_as_bytes(&leaf_id).await {
                return Some(resource_bytes);
            }
        }

        None
    }

    async fn reload(&mut self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }
}
