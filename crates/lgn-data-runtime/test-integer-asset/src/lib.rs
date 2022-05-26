use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, Resource, ResourceDescriptor,
    ResourcePathName,
};

#[resource("integer_asset")]
#[derive(Clone)]
pub struct IntegerAsset {
    pub meta: Metadata,
    pub magic_value: i32,
}

impl Asset for IntegerAsset {
    type Loader = IntegerAssetLoader;
}

#[derive(Default)]
pub struct IntegerAssetLoader {}

impl AssetLoader for IntegerAssetLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Box::new(IntegerAsset {
            meta: Metadata::new(
                ResourcePathName::default(),
                IntegerAsset::TYPENAME,
                IntegerAsset::TYPE,
            ),
            magic_value,
        }))
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {
        // nothing to do
    }
}
