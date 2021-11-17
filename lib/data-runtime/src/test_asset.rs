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
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let asset = Box::new(TestAsset { content });
        Ok(asset)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        assert!(asset.downcast_mut::<TestAsset>().is_some());
    }
}

pub(crate) mod tests {
    pub(crate) const BINARY_ASSETFILE: [u8; 39] = [
        97, 115, 102, 116, // header (asft)
        1, 0, // version
        0, 0, 0, 0, 0, 0, 0, 0, // references count (none here)
        0xb3, 0x68, 0x0, 0x81, // first asset type (RessourceType)
        1, 0, 0, 0, 0, 0, 0, 0, // assets count following in stream
        5, 0, 0, 0, 0, 0, 0, 0, // bytes for next asset data
        99, 104, 105, 108, 100, // asset data
    ];

    pub(crate) const BINARY_PARENT_ASSETFILE: [u8; 56] = [
        97, 115, 102, 116, // header (asft)
        1, 0, // version
        1, 0, 0, 0, 0, 0, 0, 0, // references count
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xb3, 0x68, 0x0,
        0x81, // first reference (RessourceId)
        0xb3, 0x68, 0x0, 0x81, // asset type (RessourceType)
        1, 0, 0, 0, 0, 0, 0, 0, // assets count following in stream
        6, 0, 0, 0, 0, 0, 0, 0, // bytes for next asset data
        112, 97, 114, 101, 110, 116, // asset data
    ];
}
