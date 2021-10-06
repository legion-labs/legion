//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.

use std::{any::Any, io};

use crate::{resource, Asset, AssetLoader, Resource};

/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
#[resource("test_asset")]
pub struct TestAsset {
    /// Test content.
    pub content: String,
}

impl Asset for TestAsset {
    type Loader = TestAssetLoader;
}

/// [`TestAsset`]'s asset creator temporarily used for testings.
///
/// To be removed once real asset types exists.
#[derive(Default)]
pub struct TestAssetLoader {}

impl AssetLoader for TestAssetLoader {
    fn load(&mut self, reader: &mut dyn std::io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let asset = Box::new(TestAsset { content });
        Ok(asset)
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
