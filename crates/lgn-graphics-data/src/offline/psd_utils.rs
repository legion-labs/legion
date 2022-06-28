//! Module providing Photoshop Document related functionality.

use crate::{runtime::RawTexture, TextureType};
use lgn_data_model::ReflectionError;

/// Photoshop Document file.
pub struct PsdFile {
    psd: psd::Psd,
}

impl PsdFile {
    /// Create a Psd from a byte stream
    /// # Errors
    /// return `ReflectionError` on failure
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ReflectionError> {
        let psd =
            psd::Psd::from_bytes(bytes).map_err(|e| ReflectionError::Generic(e.to_string()))?;
        Ok(Self { psd })
    }

    /// Returns a list of names of available layers.
    pub fn layer_list(&self) -> Vec<&str> {
        self.psd
            .layers()
            .iter()
            .map(|l| l.name())
            .collect::<Vec<_>>()
    }

    /// Creates a texture from a specified layer name.
    pub fn layer_texture(&self, name: &str) -> Option<RawTexture> {
        let layer = self.psd.layer_by_name(name)?;

        let texture = RawTexture {
            kind: TextureType::_2D,
            width: self.psd.width(),
            height: self.psd.height(),
            rgba: serde_bytes::ByteBuf::from(layer.rgba()),
        };
        Some(texture)
    }

    /// Creates a texture by merging all the psd layers.
    pub fn final_texture(&self) -> RawTexture {
        RawTexture {
            kind: TextureType::_2D,
            width: self.psd.width(),
            height: self.psd.height(),
            rgba: serde_bytes::ByteBuf::from(self.psd.rgba()),
        }
    }
}
