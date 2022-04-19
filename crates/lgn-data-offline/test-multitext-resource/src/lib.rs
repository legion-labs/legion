use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryOptions, AssetRegistryReader, HandleUntyped,
    LoadRequest, Resource, ResourceDescriptor, ResourceInstaller, ResourcePathId,
    ResourceProcessor, ResourceType, ResourceTypeAndId,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[resource("multitext_resource")]
#[derive(Serialize, Deserialize, Clone)]
pub struct MultiTextResource {
    pub text_list: Vec<String>,
}

impl MultiTextResource {
    pub fn register_type(asset_registry: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = std::sync::Arc::new(MultiTextResourceProc::default());
        asset_registry
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());
        asset_registry.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }
}

#[derive(Default)]
pub struct MultiTextResourceProc {}

#[async_trait]
impl ResourceInstaller for MultiTextResourceProc {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut buf = Vec::<u8>::new();
        reader.read_to_end(&mut buf).await?;
        let resource: MultiTextResource = serde_json::from_reader(&mut buf.as_slice()).unwrap();
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(resource))?;
        Ok(handle)
    }
}

impl ResourceProcessor for MultiTextResourceProc {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(MultiTextResource { text_list: vec![] })
    }

    fn extract_build_dependencies(&self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, AssetRegistryError> {
        let resource = resource.downcast_ref::<MultiTextResource>().unwrap();
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }
}
