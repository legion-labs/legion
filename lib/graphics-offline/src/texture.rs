//! A module providing offline texture related functionality.

use legion_data_offline::resource::{Resource, ResourceProcessor, ResourceType};
use serde::{Deserialize, Serialize};

/// `Texture` type id.
pub const TYPE_ID: ResourceType = ResourceType::new(b"offline_texture");

/// Texture type enumeration.
#[derive(Serialize, Deserialize)]
pub enum TextureType {
    /// 2d texture.
    _2D,
}

/// Offline texture resource.
#[derive(Resource, Serialize, Deserialize)]
pub struct Texture {
    /// Texture type.
    pub kind: TextureType,
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Texture pixel data.
    pub rgba: Vec<u8>,
}

/// Processor of [`Texture`]
pub struct TextureProcessor {}

impl ResourceProcessor for TextureProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Texture {
            kind: TextureType::_2D,
            width: 0,
            height: 0,
            rgba: vec![],
        })
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
        let texture = resource.downcast_ref::<Texture>().unwrap();
        serde_json::to_writer(writer, texture)?;
        Ok(1)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let texture: Texture = serde_json::from_reader(reader)?;
        Ok(Box::new(texture))
    }
}
