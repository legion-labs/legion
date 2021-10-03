use std::any::Any;

use legion_data_runtime::{AssetLoader, Asset, Resource, ResourceType};

#[derive(Resource)]
pub struct IntegerAsset {
    pub magic_value: i32,
}

impl Resource for IntegerAsset {
    const TYPENAME: &'static str = "integer_asset";
}

impl Asset for IntegerAsset {
    type Loader = IntegerAssetLoader;
}

#[derive(Default)]
pub struct IntegerAssetLoader {}

impl AssetLoader for IntegerAssetLoader {
    fn load(
        &mut self,
        _kind: ResourceType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Sync + Send>, std::io::Error> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Box::new(IntegerAsset { magic_value }))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Sync + Send)) {
        // nothing to do
    }
}
