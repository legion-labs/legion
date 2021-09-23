//! A module providing runtime texture related functionality.

use legion_data_runtime::{Asset, AssetLoader, AssetType};

/// `Texture` type id.
pub const TYPE_ID: AssetType = AssetType::new(b"runtime_texture");

/// Runtime texture.
#[derive(Asset)]
pub struct Texture {
    /// Pixel data of the image
    pub rgba: Vec<u8>,
}

/// Loader of [`Texture`].
pub struct TextureLoader {}

impl AssetLoader for TextureLoader {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let mut rgba: Vec<u8> = vec![];
        reader.read_to_end(&mut rgba)?;
        let texture = Texture { rgba };
        Ok(Box::new(texture))
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}
