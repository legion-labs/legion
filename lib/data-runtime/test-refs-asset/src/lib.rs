//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.

use std::any::Any;

use legion_data_runtime::{resource, Asset, AssetLoader, Resource};
/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
#[resource("refs_asset")]
pub struct RefsAsset {
    /// Test content.
    pub content: String,
}

impl Asset for RefsAsset {
    type Loader = RefsAssetLoader;
}

/// [`RefsAsset`]'s asset creator temporarily used for testings.
///
/// To be removed once real asset types exists.
#[derive(Default)]
pub struct RefsAssetLoader {}

impl AssetLoader for RefsAssetLoader {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, std::io::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let asset = Box::new(RefsAsset { content });
        Ok(asset)
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
