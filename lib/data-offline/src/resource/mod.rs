//! Offline management of resources.
//!

/// Types implementing `Resource` represent editor data.
pub trait Resource: Any {
    /// Cast to &dyn Any type.
    fn as_any(&self) -> &dyn Any;

    /// Cast to &mut dyn Any type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating a new resource.
    fn new_resource(&mut self) -> Box<dyn Resource>;

    /// Interface returning a list of resources that `resource` depends on for building.
    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId>;

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Resource>>;
}

mod project;
use std::any::Any;
use std::io;

use crate::asset::AssetPathId;

pub use self::project::*;

mod metadata;
pub use self::metadata::*;

mod types;
pub use self::types::*;

mod registry;
pub use self::registry::*;

mod handle;
pub use self::handle::*;

#[cfg(test)]
pub(crate) mod test_resource;
