//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.

use legion_data_runtime::{Asset, AssetDescriptor, AssetLoader, AssetType};

/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
#[derive(Asset)]
pub struct TestAsset {
    /// Test content.
    pub content: String,
}

impl AssetDescriptor for TestAsset {
    const TYPENAME: &'static str = "refs_asset";
    type Loader = TestAssetCreator;
}

/// [`TestAsset`]'s asset creator temporarily used for testings.
///
/// To be removed once real asset types exists.
#[derive(Default)]
pub struct TestAssetCreator {}

impl AssetLoader for TestAssetCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let asset = Box::new(TestAsset { content });
        Ok(asset)
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}
