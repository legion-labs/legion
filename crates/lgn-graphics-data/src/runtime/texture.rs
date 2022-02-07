//! A module providing runtime texture related functionality.

use std::{any::Any, io};

use lgn_data_model::implement_primitive_type_def;
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};

/// Runtime texture.
#[resource("runtime_texture")]
pub struct Texture {
    /// Pixel data of the image
    pub rgba: Vec<u8>,
}

impl Asset for Texture {
    type Loader = TextureLoader;
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
        let mut rgba: Vec<u8> = vec![];
        reader.read_to_end(&mut rgba)?;
        let texture = Texture { rgba };
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
