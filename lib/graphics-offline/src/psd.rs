//! Module providing Photoshop Document related functionality.

use std::any::Any;

use legion_data_offline::resource::{OfflineResource, ResourceProcessor};
use legion_data_runtime::Resource;

use crate::texture::{Texture, TextureType};

/// Photoshop Document file.
#[derive(Resource)]
pub struct PsdFile {
    content: Option<(psd::Psd, Vec<u8>)>,
}

impl Resource for PsdFile {
    const TYPENAME: &'static str = "psd";
}

impl OfflineResource for PsdFile {
    type Processor = PsdFileProcessor;
}

impl PsdFile {
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

/// A processor of Photoshop Document files.
#[derive(Default)]
pub struct PsdFileProcessor {}

impl ResourceProcessor for PsdFileProcessor {
    fn new_resource(&mut self) -> Box<dyn Any> {
        Box::new(PsdFile { content: None })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<legion_data_offline::ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let psd = resource.downcast_ref::<PsdFile>().unwrap();
        if let Some((_, content)) = &psd.content {
            writer.write_all(content).unwrap();
            Ok(content.len())
        } else {
            Ok(0)
        }
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any>> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;
        let content = if !bytes.is_empty() {
            let psd = psd::Psd::from_bytes(&bytes).map_err(|_e| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed to read .psd file")
            })?;
            Some((psd, bytes))
        } else {
            None
        };
        Ok(Box::new(PsdFile { content }))
    }
}
