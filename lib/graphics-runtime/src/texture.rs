//! A module providing runtime texture related functionality.

use std::{any::Any, io};

use legion_data_runtime::{resource, Asset, AssetLoader, Resource};

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
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut rgba: Vec<u8> = vec![];
        reader.read_to_end(&mut rgba)?;
        let texture = Texture { rgba };
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
