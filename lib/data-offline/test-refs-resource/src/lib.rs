//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper resource available.

use std::any::Any;

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};
use legion_data_runtime::Resource;

use serde::{Deserialize, Serialize};

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[derive(Resource, Serialize, Deserialize)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<ResourcePathId>,
}

impl Resource for TestResource {
    const TYPENAME: &'static str = "test_resource";
}

impl OfflineResource for TestResource {
    type Processor = TestResourceProc;
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
#[derive(Default)]
pub struct TestResourceProc {}
impl ResourceProcessor for TestResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any> {
        Box::new(TestResource {
            content: String::from("default content"),
            build_deps: vec![],
        })
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId> {
        resource
            .downcast_ref::<TestResource>()
            .unwrap()
            .build_deps
            .clone()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<TestResource>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any>> {
        let resource: TestResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
