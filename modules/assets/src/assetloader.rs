use std::{collections::HashMap, fs, io, mem, path::PathBuf};

use crate::{
    assetloader::file_format_json::{AssetFileHeader, AssetHeader, SectionHeader, MAGIC_NUMBER},
    Asset, AssetGenericHandle, AssetId, AssetType,
};

use serde::{Deserialize, Serialize};

fn asset_path(id: AssetId) -> PathBuf {
    PathBuf::from(id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: AssetId,
    secondary: AssetId,
}

/// An interface allowing to create and initialize assets.
trait AssetCreator {
    fn load(
        &mut self,
        kind: AssetType,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Asset>, io::Error>;
    fn load_init(&mut self, asset: &mut dyn Asset);
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

    pub(crate) fn load_request(&mut self, handle: AssetGenericHandle, id: AssetId) {
        self.requests.push((handle, id));
    }

    pub(crate) fn load_update(&mut self, storage: &mut impl AssetLoaderStorage) {
        for (_, id) in mem::take(&mut self.requests) {
            let file_path = asset_path(id);

            let result = match fs::File::open(file_path) {
                Ok(mut file) => match self.load_internal(&mut file) {
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

    fn load_internal(&mut self, mut file: &mut dyn io::Read) -> Result<LoadOutput, io::Error> {
        let load_dependencies = {
            let mut de = serde_json::Deserializer::from_reader(&mut file);
            let file_header = AssetFileHeader::deserialize(&mut de).unwrap();
            println!("{:?}", file_header);

            assert_eq!(file_header.magic_number, MAGIC_NUMBER);
            assert!(file_header.load_dependencies.is_empty());
            file_header.load_dependencies
        };

        // todo: for now we assume 1 section. later we need to do this for all the sections in the file.
        let assets = {
            let mut assets = vec![];
            let mut de = serde_json::Deserializer::from_reader(&mut file);
            let section_header = SectionHeader::deserialize(&mut de).unwrap();
            println!("{:?}", section_header);

            assert_eq!(section_header.section_type, 0);
            assert_eq!(section_header.asset_count, 1);

            for _ in 0..section_header.asset_count {
                let (asset_id, asset_type) = {
                    let mut de = serde_json::Deserializer::from_reader(&mut file);
                    let asset_header = AssetHeader::deserialize(&mut de).unwrap();
                    println!("{:?}", asset_header);
                    (asset_header.asset_id, asset_header.asset_id.asset_type())
                };

                // todo: better serialization format.
                // since we do not know the length of the json content to expect
                // we cannot simply skip the byts if the `creator` is not found.
                let creator = self.creators.get_mut(&asset_type).unwrap();
                let new_asset = creator.load(asset_type, file).unwrap();
                assets.push((asset_id, new_asset));
            }
            assets
        };

        Ok(LoadOutput {
            assets,
            _load_dependencies: load_dependencies,
        })
    }

    fn load_init_internal(&mut self, id: AssetId, asset: &mut dyn Asset) {
        let creator = self.creators.get_mut(&id.asset_type()).unwrap();
        creator.load_init(asset);
    }
}

pub(super) mod file_format_json {

    use serde::{Deserialize, Serialize};

    use crate::AssetId;

    use super::AssetReference;

    pub(super) const MAGIC_NUMBER: usize = 0xdeadbeef;

    #[derive(Serialize, Deserialize, Debug)]
    pub(super) struct AssetFileHeader {
        pub(super) magic_number: usize,
        pub(super) load_dependencies: Vec<AssetReference>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub(super) struct SectionHeader {
        pub(super) section_type: u8,
        pub(super) asset_count: u8,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub(super) struct AssetHeader {
        pub(super) asset_size: usize,
        pub(super) asset_id: AssetId,
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use std::{any::Any, io};

    use crate::{
        assetloader::file_format_json::{
            AssetFileHeader, AssetHeader, SectionHeader, MAGIC_NUMBER,
        },
        Asset, AssetId, AssetType,
    };

    use super::{AssetCreator, AssetLoader};

    const ASSET_TEXTURE: AssetType = AssetType::new(b"texture");

    struct TextureCreator {}

    #[derive(Debug, Serialize, Deserialize)]
    struct TextureAsset {
        content: String,
        load_state: isize,
    }

    impl Asset for TextureAsset {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    impl AssetCreator for TextureCreator {
        fn load(
            &mut self,
            _kind: AssetType,
            reader: &mut dyn io::Read,
        ) -> Result<Box<dyn Asset>, std::io::Error> {
            let mut de = serde_json::Deserializer::from_reader(reader);
            let asset = TextureAsset::deserialize(&mut de).unwrap();
            println!("{:?}", asset);
            Ok(Box::new(asset))
        }

        fn load_init(&mut self, asset: &mut dyn Asset) {
            let texture_asset = asset.as_any_mut().downcast_mut::<TextureAsset>().unwrap();
            texture_asset.load_state = 1;
        }
    }

    fn create_test_asset(mut writer: &mut [u8]) {
        let file_header = AssetFileHeader {
            magic_number: MAGIC_NUMBER,
            load_dependencies: vec![],
        };
        serde_json::to_writer_pretty(&mut writer, &file_header).unwrap();

        let section_header = SectionHeader {
            section_type: 0,
            asset_count: 1,
        };
        serde_json::to_writer_pretty(&mut writer, &section_header).unwrap();

        let asset_header = AssetHeader {
            asset_size: 64,
            asset_id: AssetId::new(ASSET_TEXTURE, 2),
        };
        serde_json::to_writer_pretty(&mut writer, &asset_header).unwrap();

        // test texture write
        let sample_texture = TextureAsset {
            content: String::from("hello_texture"),
            load_state: -1,
        };
        serde_json::to_writer(&mut writer, &sample_texture).unwrap();
    }

    #[test]
    fn asset_loading() {
        let mut loader = AssetLoader::new();

        loader
            .creators
            .insert(ASSET_TEXTURE, Box::new(TextureCreator {}));
        {
            let mut buffer = [0u8; 512];
            create_test_asset(&mut buffer[..]);
            let mut reader = &buffer[..];

            let mut output = loader.load_internal(&mut reader).unwrap();
            assert_eq!(output.assets.len(), 1);

            let preload = output.assets[0]
                .1
                .as_any()
                .downcast_ref::<TextureAsset>()
                .unwrap()
                .load_state;
            assert_eq!(preload, -1);

            for (asset_id, asset) in &mut output.assets {
                loader.load_init_internal(*asset_id, asset.as_mut());
                println!(
                    "{:?}",
                    asset.as_any().downcast_ref::<TextureAsset>().unwrap()
                );
            }

            let postload = output.assets[0]
                .1
                .as_any()
                .downcast_ref::<TextureAsset>()
                .unwrap()
                .load_state;
            assert_eq!(postload, 1);
        }
    }
}
