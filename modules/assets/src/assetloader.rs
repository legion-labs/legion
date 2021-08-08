use std::{collections::HashMap, fs, io, mem, path::PathBuf};

use crate::{Asset, AssetCreator, AssetGenericHandle, AssetId, AssetType};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

fn asset_path(id: AssetId) -> PathBuf {
    PathBuf::from(id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: AssetId,
    secondary: AssetId,
}

pub(crate) trait AssetLoaderStorage {
    fn store(&mut self, id: AssetId, asset: Result<Box<dyn Asset>, io::Error>);
}

/// The intermediate output of asset loading process.
///
/// Contains the result of loading a single file.
struct LoadOutput {
    assets: Vec<(AssetId, Box<dyn Asset>)>,
    _load_dependencies: Vec<AssetReference>,
}

/// Currently there is not threading for asset loading.
/// The `AssetLoader` needs to be polled by calling `load_update` to process
/// load requests.
pub(crate) struct AssetLoader {
    creators: HashMap<AssetType, Box<dyn AssetCreator>>,
    requests: Vec<(AssetGenericHandle, AssetId)>,
}

impl AssetLoader {
    pub(crate) fn new() -> Self {
        Self {
            creators: HashMap::new(),
            requests: vec![],
        }
    }

    pub(crate) fn register_creator(&mut self, kind: AssetType, creator: Box<dyn AssetCreator>) {
        self.creators.insert(kind, creator);
    }

    pub(crate) fn load_request(&mut self, handle: AssetGenericHandle, id: AssetId) {
        self.requests.push((handle, id));
    }

    pub(crate) fn load_update(&mut self, storage: &mut impl AssetLoaderStorage) {
        for (_, id) in mem::take(&mut self.requests) {
            let file_path = asset_path(id);

            let result = match fs::File::open(file_path) {
                Ok(mut file) => match self.load_internal(id, &mut file) {
                    Ok(mut output) => {
                        // for now we assume there is only one asset in a file
                        // and that this is the primary asset.
                        assert_eq!(output.assets.len(), 1);

                        let (asset_id, mut asset) = output.assets.pop().unwrap();
                        assert_eq!(asset_id, id);
                        self.load_init_internal(asset_id, asset.as_mut());
                        Ok(asset)
                    }
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            };

            storage.store(id, result);
        }
    }

    fn load_internal(
        &mut self,
        primary_id: AssetId,
        reader: &mut dyn io::Read,
    ) -> Result<LoadOutput, io::Error> {
        const ASSET_FILE_VERSION: u16 = 1;

        // asset file header
        let version = reader.read_u16::<LittleEndian>()?;
        if version != ASSET_FILE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Version Mismatch",
            ));
        }

        let reference_count = reader.read_u64::<LittleEndian>()?;
        let mut reference_list = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            let asset_ref =
                unsafe { std::mem::transmute::<u64, AssetId>(reader.read_u64::<LittleEndian>()?) };
            reference_list.push(AssetReference {
                primary: asset_ref,
                secondary: asset_ref,
            });
        }

        // section header
        let asset_type = unsafe {
            std::mem::transmute::<u32, AssetType>(
                reader.read_u32::<LittleEndian>().expect("valid data"),
            )
        };
        let asset_count = reader.read_u64::<LittleEndian>().expect("valid data");
        assert_eq!(asset_count, 1);

        let nbytes = reader.read_u64::<LittleEndian>().expect("valid data");

        let mut content = Vec::new();
        content.resize(nbytes as usize, 0);
        reader.read_exact(&mut content).expect("valid data");

        let creator = self.creators.get_mut(&asset_type).unwrap();
        let boxed_asset = creator.load(asset_type, &mut &content[..]).unwrap();

        Ok(LoadOutput {
            assets: vec![(primary_id, boxed_asset)],
            _load_dependencies: reference_list,
        })
    }

    fn load_init_internal(&mut self, id: AssetId, asset: &mut dyn Asset) {
        let creator = self.creators.get_mut(&id.asset_type()).unwrap();
        creator.load_init(asset);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_asset::{self, TestAsset},
        AssetId,
    };

    use super::AssetLoader;

    #[test]
    fn load_asset_file() {
        let binary_assetfile = [
            1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0,
            0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112, 97, 114, 101, 110, 116,
        ];
        let expected_content = "parent";

        let mut loader = AssetLoader::new();
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        let id = AssetId::new(test_asset::TYPE_ID, 1);
        let result = loader
            .load_internal(id, &mut &binary_assetfile[..])
            .expect("parable data");

        assert_eq!(result.assets.len(), 1);
        assert_eq!(result._load_dependencies.len(), 1);

        let (asset_id, asset) = &result.assets[0];

        let asset = asset.as_any().downcast_ref::<TestAsset>().unwrap();
        assert_eq!(asset.content, expected_content);
        assert_eq!(asset_id, &id);
    }
}
