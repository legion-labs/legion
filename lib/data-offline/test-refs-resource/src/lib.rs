//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper resource available.

use std::{any::Any, io};

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};
use legion_data_runtime::{resource, Asset, AssetLoader, Resource};
use serde::{Deserialize, Serialize};

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[resource("test_resource")]
#[derive(Serialize, Deserialize)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<ResourcePathId>,
}

impl Asset for TestResource {
    type Loader = TestResourceProc;
}

impl OfflineResource for TestResource {
    type Processor = TestResourceProc;
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
#[derive(Default)]
pub struct TestResourceProc {}

impl AssetLoader for TestResourceProc {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let resource: TestResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for TestResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
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

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
}
