use async_trait::async_trait;
use lgn_data_runtime::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[resource("text")]
#[derive(Serialize, Deserialize, Clone)]
pub struct TextResource {
    pub content: String,
}

impl TextResource {
    pub fn register_type(asset_registry: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = std::sync::Arc::new(TextResourceProc::default());
        asset_registry
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());
        asset_registry.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }

    /// # Errors
    /// return a `AssetRegistryError` if it failed to create a `TextureResource` from an async reader
    pub async fn from_reader(reader: &mut AssetRegistryReader) -> Result<Self, AssetRegistryError> {
        let mut buf = Vec::<u8>::new();
        reader.read_to_end(&mut buf).await?;
        let resource: Self = serde_json::from_reader(&mut buf.as_slice()).unwrap();
        Ok(resource)
    }
}

#[derive(Default)]
pub struct TextResourceProc {}

#[async_trait]
impl ResourceInstaller for TextResourceProc {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut buf = Vec::<u8>::new();
        reader.read_to_end(&mut buf).await?;
        let resource: TextResource = serde_json::from_reader(&mut buf.as_slice()).unwrap();
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(resource))?;
        Ok(handle)
    }
}

impl ResourceProcessor for TextResourceProc {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(TextResource {
            content: String::from("7"),
        })
    }

    fn extract_build_dependencies(&self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, AssetRegistryError> {
        let resource = resource.downcast_ref::<TextResource>().unwrap();
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }
}
