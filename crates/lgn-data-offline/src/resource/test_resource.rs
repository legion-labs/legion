//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper
//! resource available.

use std::{any::Any, io, str::FromStr};

use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};

use super::OfflineResource;
use crate::{
    resource::{ResourceProcessor, ResourceProcessorError},
    ResourcePathId,
};

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[resource("test_resource")]
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
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let mut resource = Box::new(TestResource {
            content: String::from(""),
            build_deps: vec![],
        });
        let mut buf = 0usize.to_ne_bytes();
        reader.read_exact(&mut buf[..])?;
        let length = usize::from_ne_bytes(buf);

        let mut buf = vec![0u8; length];
        reader.read_exact(&mut buf[..])?;
        resource.content = String::from_utf8(buf).unwrap();

        let mut buf = resource.build_deps.len().to_ne_bytes();
        reader.read_exact(&mut buf[..])?;
        let dep_count = usize::from_ne_bytes(buf);

        for _ in 0..dep_count {
            let mut nbytes = 0u64.to_ne_bytes();
            reader.read_exact(&mut nbytes[..])?;
            let mut buf = vec![0u8; usize::from_ne_bytes(nbytes)];
            reader.read_exact(&mut buf)?;
            resource
                .build_deps
                .push(ResourcePathId::from_str(std::str::from_utf8(&buf).unwrap()).unwrap());
        }

        Ok(resource)
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
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let resource = resource.downcast_ref::<TestResource>().unwrap();
        let mut nbytes = 0;

        let content_bytes = resource.content.as_bytes();

        let bytes = content_bytes.len().to_ne_bytes();
        nbytes += bytes.len();
        writer.write_all(&bytes)?;
        nbytes += content_bytes.len();
        writer.write_all(content_bytes)?;

        let bytes = resource.build_deps.len().to_ne_bytes();
        nbytes += bytes.len();
        writer.write_all(&bytes)?;

        for dep in &resource.build_deps {
            let str = dep.to_string();
            let str = str.as_bytes();
            let bytes = str.len().to_ne_bytes();
            writer.write_all(&bytes)?;
            nbytes += bytes.len();
            writer.write_all(str)?;
            nbytes += str.len();
        }

        Ok(nbytes)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
