use legion_content_store::{Checksum, ContentStore};
use legion_data_runtime::{ResourceId, ResourceType};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Deserialize)]
struct AssetFile {
    header: [u8; 4],
    version: u16,
    deps: Vec<(ResourceType, ResourceId)>,
    kind: ResourceType,
    assets: Vec<Vec<u8>>,
}

const ASSET_FILE_VERSION: u16 = 1;
const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";

// todo: no asset ids are written because we assume 1 asset in asset_file now.
pub fn write_assetfile(
    asset_list: impl Iterator<Item = ((ResourceType, ResourceId), Checksum)> + Clone,
    reference_list: impl Iterator<
            Item = (
                (ResourceType, ResourceId),
                ((ResourceType, ResourceId), (ResourceType, ResourceId)),
            ),
        > + Clone,
    content_store: &impl ContentStore,
) -> Result<Vec<u8>, Error> {
    // Prepare dependencies
    let mut primary_dependencies: Vec<(ResourceType, ResourceId)> =
        reference_list.map(|r| r.1 .0).collect();
    primary_dependencies.sort();
    primary_dependencies.dedup();

    let mut asset_contents = vec![];
    let mut kind: Option<ResourceType> = None;
    for content in asset_list {
        if asset_contents.is_empty() {
            kind = Some(content.0 .0);
        }
        asset_contents.push(content_store.read(content.1).unwrap());
    }

    let asset = AssetFile {
        header: *ASSET_FILE_TYPENAME,
        version: ASSET_FILE_VERSION,
        deps: primary_dependencies,
        kind: kind.unwrap(),
        assets: asset_contents,
    };

    bincode::serialize(&asset).map_err(|_e| Error::LinkFailed)
}

#[cfg(test)]
mod tests {

    use bincode::Options;
    use legion_content_store::RamContentStore;
    use legion_data_runtime::Resource;
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct RefAssetContent {
        text: &'static str,
        reference: (ResourceType, ResourceId),
    }

    fn create_ref_asset(text: &'static str, reference: ResourceId) -> Vec<u8> {
        let content = RefAssetContent {
            text,
            reference: (refs_asset::RefsAsset::TYPE, reference),
        };
        bincode::DefaultOptions::new()
            .with_varint_encoding()
            .allow_trailing_bytes()
            .serialize(&content)
            .unwrap()
    }

    #[test]
    fn one_asset_no_references() {
        let mut content_store = RamContentStore::default();

        let asset_id = (refs_asset::RefsAsset::TYPE, ResourceId::new_explicit(1));
        let asset_content = create_ref_asset("test_content", ResourceId::new_explicit(9));
        let asset_checksum = content_store.store(&asset_content).expect("to store asset");
        assert_eq!(content_store.read(asset_checksum).unwrap(), asset_content);

        let binary_assetfile = write_assetfile(
            std::iter::once((asset_id, asset_checksum)),
            std::iter::empty(),
            &content_store,
        )
        .expect("asset file");

        {
            let asset: AssetFile = bincode::deserialize_from(&binary_assetfile[..]).unwrap();
            assert_eq!(&asset.header, ASSET_FILE_TYPENAME);
            assert_eq!(asset.version, ASSET_FILE_VERSION);
            assert_eq!(asset.deps.len(), 0);
            assert_eq!(asset.kind, refs_asset::RefsAsset::TYPE);
            assert_eq!(asset.assets.len(), 1);
            assert_eq!(&asset.assets[0], &asset_content);
        }
    }

    #[test]
    fn two_dependent_assets() {
        let mut content_store = RamContentStore::default();

        let child_id = (refs_asset::RefsAsset::TYPE, ResourceId::new_explicit(1));
        let child_content = create_ref_asset("child", ResourceId::new_explicit(9));
        let child_checksum = content_store.store(&child_content).expect("to store asset");
        assert_eq!(content_store.read(child_checksum).unwrap(), child_content);

        let parent_id = (refs_asset::RefsAsset::TYPE, ResourceId::new_explicit(2));
        let parent_content = create_ref_asset("parent", ResourceId::new_explicit(1));
        let parent_checksum = content_store
            .store(&parent_content)
            .expect("to store asset");
        assert_eq!(content_store.read(parent_checksum).unwrap(), parent_content);

        let reference_list = vec![(parent_id, (child_id, child_id))];

        let parent_assetfile = write_assetfile(
            std::iter::once((parent_id, parent_checksum)),
            reference_list.iter().copied(),
            &content_store,
        )
        .expect("asset file");

        let _child_assetfile = write_assetfile(
            std::iter::once((child_id, child_checksum)),
            std::iter::empty(),
            &content_store,
        )
        .expect("asset file");

        //println!("{:?} : {:?}", parent_id, parent_assetfile);
        //println!("{:?} : {:?}", child_id, _child_assetfile);

        {
            let asset: AssetFile = bincode::deserialize_from(&parent_assetfile[..]).unwrap();
            assert_eq!(&asset.header, ASSET_FILE_TYPENAME);
            assert_eq!(asset.version, ASSET_FILE_VERSION);
            assert_eq!(asset.deps.len(), reference_list.len());

            #[allow(clippy::needless_range_loop)]
            for i in 0..reference_list.len() {
                let (_, (primary_ref, secondary_ref)) = reference_list[i];
                let dep_id = asset.deps[i];
                assert_eq!(dep_id, primary_ref);
                assert_eq!(dep_id, secondary_ref);
            }

            assert_eq!(asset.kind, refs_asset::RefsAsset::TYPE);
            assert_eq!(asset.assets.len(), 1);
            assert_eq!(&asset.assets[0], &parent_content);
        }
    }
}
