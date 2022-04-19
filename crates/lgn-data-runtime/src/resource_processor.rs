use std::io;

use crate::{AssetRegistryError, Resource, ResourcePathId};

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor: Send + Sync {
    /// Interface returning a resource in a default state. Useful when creating
    /// a new resource.
    fn new_resource(&self) -> Box<dyn Resource>;

    /// Interface returning a list of resources that `resource` depends on for
    /// building.
    fn extract_build_dependencies(&self, resource: &dyn Resource) -> Vec<ResourcePathId>;

    /// Interface defining serialization behavior of the resource.
    /// # Errors
    /// Will return `AssetRegistryError` if the resource was not written properly
    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn io::Write,
    ) -> Result<usize, AssetRegistryError>;
}
