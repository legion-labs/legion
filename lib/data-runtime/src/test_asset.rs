//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.

use crate::{Asset, AssetLoadResult, AssetType};

/// Type id of test asset.
pub const TYPE_ID: AssetType = AssetType::new(b"test_asset");

/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
pub struct TestAsset {
    /// Test content.
    pub content: String,
}

impl Asset for TestAsset {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// [`TestAsset`]'s asset creator temporarily used for testings.
///
/// To be removed once real asset types exists.
pub fn load_test_asset(_kind: AssetType, reader: &mut dyn std::io::Read) -> AssetLoadResult {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let asset = Box::new(TestAsset { content });
    Ok(asset)
}
