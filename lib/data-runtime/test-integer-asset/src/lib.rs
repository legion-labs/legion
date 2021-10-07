use std::{any::Any, io};

use legion_data_runtime::{resource, Asset, AssetLoader, Resource};

#[resource("integer_asset")]
pub struct IntegerAsset {
    pub magic_value: i32,
}

impl Asset for IntegerAsset {
    type Loader = IntegerAssetLoader;
}

#[derive(Default)]
pub struct IntegerAssetLoader {}

impl AssetLoader for IntegerAssetLoader {
    fn load(&mut self, reader: &mut dyn std::io::Read) -> io::Result<Box<dyn Any + Sync + Send>> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Box::new(IntegerAsset { magic_value }))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Sync + Send)) {
        // nothing to do
    }
}
