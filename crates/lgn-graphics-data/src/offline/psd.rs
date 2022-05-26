//! Module providing Photoshop Document related functionality.

use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathName, ResourceProcessor, ResourceProcessorError,
};

use crate::offline_texture::{Texture, TextureType};

/// Photoshop Document file.
#[resource("psd")]
pub struct PsdFile {
    meta: Metadata,
    content: Option<(psd::Psd, Vec<u8>)>,
}

impl Asset for PsdFile {
    type Loader = PsdFileProcessor;
}

impl OfflineResource for PsdFile {
    type Processor = PsdFileProcessor;
}

impl PsdFile {
    /// Creates a Photoshop Document from byte array.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match psd::Psd::from_bytes(bytes) {
            Ok(psd) => Some(Self {
                meta: Metadata::new(ResourcePathName::default(), Self::TYPENAME, Self::TYPE),
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
            meta: Metadata::new(
                ResourcePathName::default(),
                Texture::TYPENAME,
                Texture::TYPE,
            ),
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
            meta: Metadata::new(
                ResourcePathName::default(),
                Texture::TYPENAME,
                Texture::TYPE,
            ),
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
        Self {
            meta: Metadata::new(ResourcePathName::default(), Self::TYPENAME, Self::TYPE),
            content: self
                .content
                .as_ref()
                .map(|(_, bytes)| (psd::Psd::from_bytes(bytes).unwrap(), bytes.clone())),
        }
    }
}

/// A processor of Photoshop Document files.
#[derive(Default)]
pub struct PsdFileProcessor {}

impl AssetLoader for PsdFileProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;
        let content = if bytes.is_empty() {
            None
        } else {
            let psd = psd::Psd::from_bytes(&bytes).map_err(|_e| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed to read .psd file")
            })?;
            Some((psd, bytes))
        };
        Ok(Box::new(PsdFile {
            meta: Metadata::new(
                ResourcePathName::default(),
                PsdFile::TYPENAME,
                PsdFile::TYPE,
            ),
            content,
        }))
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {}
}

impl ResourceProcessor for PsdFileProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(PsdFile {
            meta: Metadata::new(
                ResourcePathName::default(),
                PsdFile::TYPENAME,
                PsdFile::TYPE,
            ),
            content: None,
        })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Resource,
    ) -> Vec<lgn_data_runtime::ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let psd = resource.downcast_ref::<PsdFile>().unwrap();
        if let Some((_, content)) = &psd.content {
            writer.write_all(content).unwrap();
            Ok(content.len())
        } else {
            Ok(0)
        }
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
