//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is responsible for their storage - which includes both on-disk storage and source control interactions.
//!
//! [`ResourceRegistry`] takes responsibility of managing the in-memory representation of resources.

use std::any::Any;
use std::io;

use legion_data_runtime::Asset;

use crate::PropertyDescriptor;
use crate::ResourcePathId;

/// The trait defines a resource that can be stored in a [`Project`].
pub trait OfflineResource: Asset {
    /// Offline resource processor bound to the resource.
    type Processor: ResourceProcessor + Send + Sync + Default + 'static;
}

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating a new resource.
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync>;

    /// Interface returning a list of resources that `resource` depends on for building.
    fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId>;

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> io::Result<Box<dyn Any + Send + Sync>>;

    /// Interface defining the resource properties
    fn get_resource_properties(
        &self,
        _resource: &dyn Any,
    ) -> Result<Vec<PropertyDescriptor>, &'static str> {
        Ok(vec![])
    }

    /// Interface defining field serialization by name
    fn write_property(
        &self,
        _resource: &mut dyn Any,
        _field_name: &str,
        _field_value: &str,
    ) -> Result<(), &'static str> {
        Err("write_property not implemented")
    }
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

mod resource_handles;
pub use self::resource_handles::*;

#[cfg(test)]
pub(crate) mod test_resource;
