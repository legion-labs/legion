//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper resource available.

use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor, ResourceType},
};

use serde::{Deserialize, Serialize};

/// Type id of test resource. Used until we have proper resource types.
pub const TYPE_ID: ResourceType = ResourceType::new(b"test_resource");

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[derive(Resource, Serialize, Deserialize)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<AssetPathId>,
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
pub struct TestResourceProc {}
impl ResourceProcessor for TestResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TestResource {
            content: String::from("default content"),
            build_deps: vec![],
        })
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId> {
        resource
            .downcast_ref::<TestResource>()
            .unwrap()
            .build_deps
            .clone()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<TestResource>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: TestResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
