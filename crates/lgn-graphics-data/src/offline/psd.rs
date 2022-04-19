//! Module providing Photoshop Document related functionality.

use std::sync::Arc;

use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryOptions, AssetRegistryReader, HandleUntyped,
    LoadRequest, Resource, ResourceDescriptor, ResourceInstaller, ResourceProcessor, ResourceType,
    ResourceTypeAndId,
};
use tokio::io::AsyncReadExt;

use crate::offline_texture::{Texture, TextureType};

/// Photoshop Document file.
#[resource("psd")]
pub struct PsdFile {
    content: Option<(psd::Psd, Vec<u8>)>,
}

impl PsdFile {
    pub fn register_type(asset_registry: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        let installer = Arc::new(PsdFileProcessor::default());
        asset_registry
            .add_resource_installer(<Self as ResourceDescriptor>::TYPE, installer.clone());
        asset_registry.add_processor(<Self as ResourceDescriptor>::TYPE, installer);
    }

    /// Creates a Photoshop Document from byte array.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match psd::Psd::from_bytes(bytes) {
            Ok(psd) => Some(Self {
                content: Some((psd, bytes.to_vec())),
            }),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }

    /// Returns a list of names of available layers.
    pub fn layer_list(&self) -> Option<Vec<&str>> {
        let (psd, _) = self.content.as_ref()?;
        Some(psd.layers().iter().map(|l| l.name()).collect::<Vec<_>>())
    }

    /// Creates a texture from a specified layer name.
    pub fn layer_texture(&self, name: &str) -> Option<Texture> {
        let (psd, _) = self.content.as_ref()?;
        let layer = psd.layer_by_name(name)?;

        let texture = Texture {
            kind: TextureType::_2D,
            width: psd.width(),
            height: psd.height(),
            rgba: layer.rgba(),
        };
        Some(texture)
    }

    /// Creates a texture by merging all the psd layers.
    pub fn final_texture(&self) -> Option<Texture> {
        let (psd, _) = self.content.as_ref()?;

        let texture = Texture {
            kind: TextureType::_2D,
            width: psd.width(),
            height: psd.height(),
            rgba: psd.rgba(),
        };

        Some(texture)
    }
}

impl Clone for PsdFile {
    fn clone(&self) -> Self {
        match &self.content {
            Some((_, bytes)) => Self {
                content: Some((psd::Psd::from_bytes(bytes).unwrap(), bytes.clone())),
            },
            None => Self { content: None },
        }
    }
}

/// A processor of Photoshop Document files.
#[derive(Default)]
struct PsdFileProcessor {}

#[async_trait]
impl ResourceInstaller for PsdFileProcessor {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let content = if bytes.is_empty() {
            None
        } else {
            let psd = psd::Psd::from_bytes(&bytes).map_err(|_e| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed to read .psd file")
            })?;
            Some((psd, bytes))
        };
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(PsdFile { content }))?;
        Ok(handle)
    }
}

impl ResourceProcessor for PsdFileProcessor {
    fn new_resource(&self) -> Box<dyn Resource> {
        Box::new(PsdFile { content: None })
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
        let psd = resource.downcast_ref::<PsdFile>().unwrap();
        if let Some((_, content)) = &psd.content {
            writer.write_all(content).unwrap();
            Ok(content.len())
        } else {
            Ok(0)
        }
    }
}
