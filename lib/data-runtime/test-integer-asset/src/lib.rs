use legion_data_runtime::{Asset, AssetLoader, AssetType};

/// Type id of test asset.
pub const TYPE_ID: AssetType = AssetType::new(b"integer_asset");

pub struct IntegerAsset {
    pub magic_value: i32,
}

impl Asset for IntegerAsset {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct IntegerAssetLoader {}

impl AssetLoader for IntegerAssetLoader {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Sync + Send>, std::io::Error> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Box::new(IntegerAsset { magic_value }))
    }
}
