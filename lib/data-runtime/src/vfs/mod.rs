use crate::ResourceId;

// todo: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
pub(crate) trait Device: Send {
    fn lookup(&self, id: ResourceId) -> Option<Vec<u8>>;
}

mod build_device;
mod cas_device;
mod dir_device;

pub(crate) use build_device::*;
pub(crate) use cas_device::*;
pub(crate) use dir_device::*;
