use std::sync::Arc;

use async_trait::async_trait;
use lgn_content_store::{Identifier, Provider};
use lgn_tracing::error;

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Content addressable storage device. Resources are accessed through a
/// manifest access table.
pub(crate) struct CasDevice {
    manifest: Option<Manifest>,
    provider: Arc<Provider>,
}

impl CasDevice {
    pub(crate) fn new(manifest: Option<Manifest>, provider: Arc<Provider>) -> Self {
        Self { manifest, provider }
    }
}

#[async_trait]
impl Device for CasDevice {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Some(manifest) = &self.manifest {
            let checksum = manifest.find(type_id)?;
            let content = self.provider.read(&checksum).await.ok()?;
            Some(content)
        } else {
            None
        }
    }

    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>> {
        None
    }

    async fn reload_manifest(&mut self, manifest_id: &Identifier) {
        if let Ok(content) = self.provider.read(manifest_id).await {
            match serde_json::from_reader::<_, Manifest>(content.as_slice()) {
                Ok(manifest) => self.manifest = Some(manifest),
                Err(error) => error!("failed to read manifest contents: {}", error),
            }
        }
    }
}
