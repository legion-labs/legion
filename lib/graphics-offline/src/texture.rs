//! A module providing offline texture related functionality.

use std::{any::Any, io};

use lgn_data_offline::resource::{OfflineResource, ResourceProcessor};
use lgn_data_runtime::{resource, Asset, AssetLoader, Resource};
use serde::{Deserialize, Serialize};

/// Texture type enumeration.
#[derive(Serialize, Deserialize)]
pub enum TextureType {
    /// 2d texture.
    _2D,
}

/// Offline texture resource.
#[resource("offline_texture")]
#[derive(Serialize, Deserialize)]
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

impl Asset for Texture {
    type Loader = TextureProcessor;
}

impl OfflineResource for Texture {
    type Processor = TextureProcessor;
}

/// Processor of [`Texture`]
#[derive(Default)]
pub struct TextureProcessor {}

impl AssetLoader for TextureProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let texture: Texture = bincode::deserialize_from(reader).unwrap();
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for TextureProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(Texture {
            kind: TextureType::_2D,
            width: 0,
            height: 0,
            rgba: vec![],
        })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let texture = resource.downcast_ref::<Texture>().unwrap();
        bincode::serialize_into(writer, texture).unwrap();
        Ok(1)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
}
