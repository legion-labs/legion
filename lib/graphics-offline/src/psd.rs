//! Module providing Photoshop Document related functionality.

use legion_data_offline::resource::{Resource, ResourceProcessor};
use legion_data_runtime::ResourceType;

use crate::texture::{Texture, TextureType};

/// `PsdFile` type id.
pub const TYPE_ID: ResourceType = ResourceType::new(b"psd");

/// Photoshop Document file.
#[derive(Resource)]
pub struct PsdFile {
    content: Option<Vec<u8>>,
}

impl PsdFile {
    /// Creates a Photoshop Document from byte array.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match psd::Psd::from_bytes(bytes) {
            Ok(_) => Some(Self {
                content: Some(bytes.to_vec()),
            }),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }

    /// Creates a texture by merging all the psd layers.
    pub fn final_texture(&self) -> Option<Texture> {
        let content = self.content.as_ref()?;
        let psd = psd::Psd::from_bytes(content).ok()?;

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
pub struct PsdFileProcessor {}

impl ResourceProcessor for PsdFileProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(PsdFile { content: None })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Resource,
    ) -> Vec<legion_data_offline::asset::AssetPathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let psd = resource.downcast_ref::<PsdFile>().unwrap();
        if let Some(content) = &psd.content {
            writer.write_all(content).unwrap();
            Ok(content.len())
        } else {
            Ok(0)
        }
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;
        let content = if !bytes.is_empty() {
            let _psd = psd::Psd::from_bytes(&bytes).map_err(|_e| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed to read .psd file")
            })?;
            Some(bytes)
        } else {
            None
        };
        Ok(Box::new(PsdFile { content }))
    }
}
