use async_trait::async_trait;

use crate::ResourceTypeAndId;

// FIXME: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
/// Device that can load/reload resources
#[async_trait]
pub trait Device: Send + Sync {
    /// Load a resource
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>>;
    /// Reload a resource
    async fn reload(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>>;
}

mod build_device;
mod cas_device;

pub(crate) use build_device::*;
pub(crate) use cas_device::*;
