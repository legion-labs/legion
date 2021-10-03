//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is responsible for their storage - which includes both on-disk storage and source control interactions.
//!
//! [`ResourceRegistry`] takes responsibility of managing the in-memory representation of resources.

use std::any::Any;
use std::io;

use crate::ResourcePathId;

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating a new resource.
    fn new_resource(&mut self) -> Box<dyn Any>;

    /// Interface returning a list of resources that `resource` depends on for building.
    fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId>;

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any>>;
}

mod project;

pub use self::project::*;

mod metadata;
pub use self::metadata::*;

mod path_name;
pub use self::path_name::*;

mod registry;
pub use self::registry::*;

mod handle;
pub use self::handle::*;

#[cfg(test)]
pub(crate) mod test_resource;
