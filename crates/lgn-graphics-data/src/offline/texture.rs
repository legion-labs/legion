//! A module providing offline texture related functionality.

use std::sync::Arc;

use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryOptions, AssetRegistryReader, HandleUntyped,
    LoadRequest, Resource, ResourceDescriptor, ResourceInstaller, ResourceProcessor, ResourceType,
    ResourceTypeAndId,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

/// Texture type enumeration.
#[derive(Serialize, Deserialize, Clone)]
pub enum TextureType {
    /// 2d texture.
    _2D,
}

/// Offline texture resource.
#[resource("offline_texture")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Texture {
    /// Texture type.
    pub kind: TextureType,
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Texture pixel data.
    #[serde(with = "serde_bytes")]
    pub rgba: Vec<u8>,
}

impl Texture {
    pub fn register_type(asset_registry: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = Arc::new(TextureProcessor::default());
        asset_registry
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());
        asset_registry.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }
}
/// Processor of [`Texture`]
#[derive(Default)]
pub struct TextureProcessor {}

#[async_trait]
impl ResourceInstaller for TextureProcessor {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;

        let texture: Texture = bincode::deserialize_from(&mut buffer.as_slice()).unwrap();
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(texture))?;
        Ok(handle)
    }
}

impl ResourceProcessor for TextureProcessor {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(Texture {
            kind: TextureType::_2D,
            width: 0,
            height: 0,
            rgba: vec![],
        })
    }

    fn extract_build_dependencies(
        &self,
        _resource: &dyn Resource,
    ) -> Vec<lgn_data_runtime::ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, AssetRegistryError> {
        let texture = resource.downcast_ref::<Texture>().unwrap();
        bincode::serialize_into(writer, texture).unwrap();
        Ok(1)
    }
}
