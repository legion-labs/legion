//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper
//! resource available.

use std::str::FromStr;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;

use crate::{
    resource, AssetRegistryError, AssetRegistryReader, HandleUntyped, LoadRequest, Resource,
    ResourceInstaller, ResourcePathId, ResourceProcessor, ResourceTypeAndId,
};
extern crate self as lgn_data_runtime;

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[resource("test_resource")]
#[derive(Clone)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<ResourcePathId>,
}

impl Default for TestResource {
    fn default() -> Self {
        Self {
            content: String::from("default content"),
            build_deps: vec![],
        }
    }
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
#[derive(Default)]
struct TestResourceProc {}

#[async_trait]
impl ResourceInstaller for TestResourceProc {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut resource = Box::new(TestResource {
            content: String::from(""),
            build_deps: vec![],
        });
        let mut buf = 0usize.to_ne_bytes();
        reader.read_exact(&mut buf[..]).await?;
        let length = usize::from_ne_bytes(buf);

        let mut buf = vec![0u8; length];
        reader.read_exact(&mut buf[..]).await?;
        resource.content = String::from_utf8(buf).unwrap();

        let mut buf = resource.build_deps.len().to_ne_bytes();
        reader.read_exact(&mut buf[..]).await?;
        let dep_count = usize::from_ne_bytes(buf);

        for _ in 0..dep_count {
            let mut nbytes = 0u64.to_ne_bytes();
            reader.read_exact(&mut nbytes[..]).await?;
            let mut buf = vec![0u8; usize::from_ne_bytes(nbytes)];
            reader.read_exact(&mut buf).await?;
            resource
                .build_deps
                .push(ResourcePathId::from_str(std::str::from_utf8(&buf).unwrap()).unwrap());
        }

        let handle = request.asset_registry.set_resource(resource_id, resource)?;
        Ok(handle)
    }
}

impl ResourceProcessor for TestResourceProc {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(TestResource {
            content: String::from("default content"),
            build_deps: vec![],
        })
    }

    fn extract_build_dependencies(&self, resource: &dyn Resource) -> Vec<ResourcePathId> {
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
    ) -> Result<usize, AssetRegistryError> {
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
}
