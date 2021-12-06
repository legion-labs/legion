use crate::ResourceId;

// todo: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
pub(crate) trait Device: Send {
    fn load(&self, id: ResourceId) -> Option<Vec<u8>>;
    fn reload(&self, _id: ResourceId) -> Option<Vec<u8>> {
        None
    }
}

mod build_device;
mod cas_device;
mod dir_device;

pub(crate) use build_device::*;
pub(crate) use cas_device::*;
pub(crate) use dir_device::*;
