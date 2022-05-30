//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper
//! resource available.

use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathId, ResourcePathName, ResourceProcessor,
    ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[resource("test_resource")]
#[derive(Serialize, Deserialize, Clone)]
pub struct TestResource {
    pub meta: Metadata,
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
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let resource: TestResource = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {}
}

impl ResourceProcessor for TestResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TestResource {
            meta: Metadata::new(
                ResourcePathName::default(),
                TestResource::TYPENAME,
                TestResource::TYPE,
            ),
            content: String::from("default content"),
            build_deps: vec![],
        })
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<ResourcePathId> {
        resource
            .downcast_ref::<TestResource>()
            .unwrap()
            .build_deps
            .clone()
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let resource = resource.downcast_ref::<TestResource>().unwrap();
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
