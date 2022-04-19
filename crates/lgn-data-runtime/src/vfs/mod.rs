use async_trait::async_trait;
use lgn_content_store::Identifier;

use crate::{AssetRegistryReader, ResourceTypeAndId};

// FIXME: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
#[async_trait]
pub(crate) trait Device: Send + Sync {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>>;
    async fn get_reader(&self, type_id: ResourceTypeAndId) -> Option<AssetRegistryReader>;
    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>>;
    async fn reload_manifest(&mut self, _manifest_id: &Identifier) {}
}

mod build_device;
mod cas_device;
mod dir_device;

pub(crate) use build_device::*;
pub(crate) use cas_device::*;
pub(crate) use dir_device::*;
