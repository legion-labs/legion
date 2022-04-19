use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryOptions, AssetRegistryReader, HandleUntyped,
    LoadRequest, Resource, ResourceDescriptor, ResourceInstaller, ResourcePathId,
    ResourceProcessor, ResourceType, ResourceTypeAndId,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[resource("bin")]
#[derive(Serialize, Deserialize, Clone)]
pub struct BinaryResource {
    pub content: Vec<u8>,
}

impl BinaryResource {
    pub fn register_type(asset_registry: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = std::sync::Arc::new(BinaryResourceProc::default());
        asset_registry
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());
        asset_registry.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }
}

#[derive(Default)]
pub struct BinaryResourceProc {}

#[async_trait]
impl ResourceInstaller for BinaryResourceProc {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut resource = BinaryResource { content: vec![] };
        reader.read_to_end(&mut resource.content).await?;
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(resource))?;
        Ok(handle)
    }
}

impl ResourceProcessor for BinaryResourceProc {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(BinaryResource { content: vec![] })
    }

    fn extract_build_dependencies(&self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, AssetRegistryError> {
        let resource = resource.downcast_ref::<BinaryResource>().unwrap();
        writer.write_all(&resource.content)?;
        Ok(1) // no bytes written exposed by serde.
    }
}
