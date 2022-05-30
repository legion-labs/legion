//! Module providing Photoshop Document related functionality.

use std::io;

use lgn_data_offline::resource::RawContent;
use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathName, ResourceProcessor, ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

use crate::offline_texture::{Texture, TextureType};

/// Photoshop Document file.
#[resource("psd")]
#[derive(Serialize, Deserialize)]
pub struct PsdFile {
    meta: Metadata,

    #[serde(skip)]
    psd: Option<psd::Psd>,

    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

impl Asset for PsdFile {
    type Loader = PsdFileProcessor;
}

impl OfflineResource for PsdFile {
    type Processor = PsdFileProcessor;
}

impl RawContent for PsdFile {
    fn set_raw_content(&mut self, data: &[u8]) {
        self.data = data.to_vec();

        let psd = psd::Psd::from_bytes(&self.data).map_err(|e| {
            ResourceProcessorError::ResourceSerializationFailed(Self::TYPENAME, e.to_string())
        }).unwrap();

        self.psd = Some(psd);
    }
}

impl PsdFile {
    /// Creates a Photoshop Document from byte array.
    /// # Errors
    /// Will return an error if unable to deserialize the psd data.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ResourceProcessorError> {
        let psd = psd::Psd::from_bytes(bytes).map_err(|e| {
            ResourceProcessorError::ResourceSerializationFailed(Self::TYPENAME, e.to_string())
        })?;
        Ok(Self {
            meta: Metadata::new(ResourcePathName::default(), Self::TYPENAME, Self::TYPE),
            psd: Some(psd),
            data: bytes.to_vec(),
        })
    }

    /// Returns a list of names of available layers.
    pub fn layer_list(&self) -> Option<Vec<&str>> {
        let psd = self.psd.as_ref()?;
        Some(psd.layers().iter().map(|l| l.name()).collect::<Vec<_>>())
    }

    /// Creates a texture from a specified layer name.
    pub fn layer_texture(&self, name: &str) -> Option<Texture> {
        let psd = self.psd.as_ref()?;
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
        let psd = self.psd.as_ref()?;

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
            psd: Some(psd::Psd::from_bytes(&self.data).unwrap()),
            data: self.data.clone(),
        }
    }
}

/// A processor of Photoshop Document files.
#[derive(Default)]
pub struct PsdFileProcessor {}

impl AssetLoader for PsdFileProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let mut resource: PsdFile = serde_json::from_reader(reader)?;
        resource.psd = Some(
            psd::Psd::from_bytes(&resource.data)
                .map_err(|e| AssetLoaderError::ErrorLoading(PsdFile::TYPENAME, e.to_string()))?,
        );
        Ok(Box::new(resource))
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
            psd: None,
            data: vec![],
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
        serde_json::to_writer_pretty(writer, psd)?;
        Ok(1)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
