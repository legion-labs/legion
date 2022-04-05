use crate::ResourceTypeAndId;
use async_trait::async_trait;

// FIXME: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
#[async_trait]
pub(crate) trait Device: Send + Sync {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>>;
    async fn reload(&self, _: ResourceTypeAndId) -> Option<Vec<u8>>;
}

mod build_device;
mod cas_device;
mod dir_device;

pub(crate) use build_device::*;
pub(crate) use cas_device::*;
pub(crate) use dir_device::*;
