use assets::{Asset, AssetCreator, AssetType};

/// Type id of test asset.
pub const TYPE_ID: AssetType = AssetType::new(b"mock_asset");

pub struct MockAsset {
    pub magic_value: i32,
}

impl Asset for MockAsset {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct MockAssetCreator {}

impl AssetCreator for MockAssetCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset>, std::io::Error> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Box::new(MockAsset { magic_value }))
    }

    fn load_init(&mut self, _asset: &mut dyn Asset) {
        // nothing to do
    }
}
