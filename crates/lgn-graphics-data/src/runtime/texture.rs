//! A module providing runtime texture related functionality.

use std::{any::Any, io};

use lgn_data_model::implement_primitive_type_def;
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};
use serde::{Deserialize, Serialize};

use crate::{encode_mip_chain_from_offline_texture, TextureFormat};

/// Runtime texture.
#[resource("runtime_texture")]
#[derive(Serialize, Deserialize)]
pub struct Texture {
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Desired HW texture format
    pub format: TextureFormat,
    /// Mip chain pixel data of the image in hardware encoded form
    pub texture_data: Vec<Vec<u8>>,
}

impl Asset for Texture {
    type Loader = TextureLoader;
}

impl Texture {
    pub fn compile_from_offline(
        width: u32,
        height: u32,
        format: TextureFormat,
        alpha_blended: bool,
        rgba: &[u8],
        writer: &mut dyn std::io::Write,
    ) {
        let texture = Self {
            width,
            height,
            format,
            texture_data: encode_mip_chain_from_offline_texture(
                width,
                height,
                format,
                alpha_blended,
                rgba,
            ),
        };
        bincode::serialize_into(writer, &texture).unwrap();
    }
}

/// Reference Type for Texture
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TextureReferenceType(lgn_data_runtime::Reference<Texture>);
impl TextureReferenceType {
    /// Expose internal id
    pub fn id(&self) -> lgn_data_runtime::ResourceTypeAndId {
        self.0.id()
    }
}
implement_primitive_type_def!(TextureReferenceType);

/// Loader of [`Texture`].
#[derive(Default)]
pub struct TextureLoader {}

impl AssetLoader for TextureLoader {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let texture: Texture = bincode::deserialize_from(reader).unwrap();
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
