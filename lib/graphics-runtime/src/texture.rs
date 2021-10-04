//! A module providing runtime texture related functionality.

use std::any::Any;

use legion_data_runtime::{resource, Asset, AssetLoader, Resource, ResourceType};

/// Runtime texture.
#[resource("runtime_texture")]
pub struct Texture {
    /// Pixel data of the image
    pub rgba: Vec<u8>,
}

impl Asset for Texture {
    type Loader = TextureLoader;
}

/// Loader of [`Texture`].
#[derive(Default)]
pub struct TextureLoader {}

impl AssetLoader for TextureLoader {
    fn load(
        &mut self,
        _kind: ResourceType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, std::io::Error> {
        let mut rgba: Vec<u8> = vec![];
        reader.read_to_end(&mut rgba)?;
        let texture = Texture { rgba };
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
