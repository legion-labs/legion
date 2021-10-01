//! A module providing runtime texture related functionality.

use legion_data_runtime::{Asset, AssetDescriptor, AssetLoader, ResourceType};

/// Runtime texture.
#[derive(Asset)]
pub struct Texture {
    /// Pixel data of the image
    pub rgba: Vec<u8>,
}

impl AssetDescriptor for Texture {
    const TYPENAME: &'static str = "runtime_texture";
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
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let mut rgba: Vec<u8> = vec![];
        reader.read_to_end(&mut rgba)?;
        let texture = Texture { rgba };
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}
