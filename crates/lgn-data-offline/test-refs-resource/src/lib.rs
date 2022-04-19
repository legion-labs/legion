//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper
//! resource available.

use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryOptions, AssetRegistryReader, HandleUntyped,
    LoadRequest, Resource, ResourceDescriptor, ResourceInstaller, ResourcePathId,
    ResourceProcessor, ResourceType, ResourceTypeAndId,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
#[resource("test_resource")]
#[derive(Serialize, Deserialize, Clone)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<ResourcePathId>,
}

impl TestResource {
    pub fn register_type(asset_registry_options: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = std::sync::Arc::new(TestResourceProc::default());
        asset_registry_options
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());

        asset_registry_options.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
#[derive(Default)]
pub struct TestResourceProc {}

#[async_trait]
impl ResourceInstaller for TestResourceProc {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut buf = Vec::<u8>::new();
        reader.read_to_end(&mut buf).await?;
        let resource: TestResource = serde_json::from_reader(&mut buf.as_slice()).unwrap();
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(resource))?;
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
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }
}
