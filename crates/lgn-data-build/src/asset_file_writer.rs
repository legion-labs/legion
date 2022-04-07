use lgn_content_store::{ContentProvider, ContentReaderExt, Identifier};
use lgn_data_runtime::{ResourceType, ResourceTypeAndId};
use lgn_tracing::async_span_scope;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Deserialize)]
struct AssetFile {
    header: [u8; 4],
    version: u16,
    deps: Vec<ResourceTypeAndId>,
    kind: ResourceType,
    assets: Vec<serde_bytes::ByteBuf>,
}

const ASSET_FILE_VERSION: u16 = 1;
const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";

// todo: no asset ids are written because we assume 1 asset in asset_file now.
pub async fn write_assetfile(
    asset_list: impl Iterator<Item = (ResourceTypeAndId, Identifier)> + Clone,
    reference_list: impl Iterator<Item = (ResourceTypeAndId, (ResourceTypeAndId, ResourceTypeAndId))>
        + Clone,
    content_store: &(dyn ContentProvider + Send + Sync),
) -> Result<Vec<u8>, Error> {
    async_span_scope!("write_assetfile");

    // Prepare dependencies
    let mut primary_dependencies: Vec<ResourceTypeAndId> = reference_list.map(|r| r.1 .0).collect();
    primary_dependencies.sort();
    primary_dependencies.dedup();

    let mut asset_contents = vec![];
    let mut kind: Option<ResourceType> = None;
    {
        async_span_scope!("content_store_read");
        for content in asset_list {
            if asset_contents.is_empty() {
                kind = Some(content.0.kind);
            }
            asset_contents.push(content_store.read_content(&content.1).await.unwrap());
        }
    }

    let asset = AssetFile {
        header: *ASSET_FILE_TYPENAME,
        version: ASSET_FILE_VERSION,
        deps: primary_dependencies,
        kind: kind.unwrap(),
        assets: asset_contents
            .into_iter()
            .map(serde_bytes::ByteBuf::from)
            .collect::<Vec<_>>(),
    };

    {
        async_span_scope!("bincode::serialize");
        bincode::serialize(&asset).map_err(|_e| Error::LinkFailed)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use bincode::Options;
    use lgn_content_store::{ContentWriterExt, MemoryProvider};
    use lgn_data_runtime::{Resource, ResourceId};
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct RefAssetContent {
        text: &'static str,
        reference: ResourceTypeAndId,
    }

    fn create_ref_asset(text: &'static str, reference: ResourceId) -> Vec<u8> {
        let content = RefAssetContent {
            text,
            reference: ResourceTypeAndId {
                kind: refs_asset::RefsAsset::TYPE,
                id: reference,
            },
        };
        bincode::DefaultOptions::new()
            .with_varint_encoding()
            .allow_trailing_bytes()
            .serialize(&content)
            .unwrap()
    }

    #[tokio::test]
    async fn one_asset_no_references() {
        let content_store: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));

        let asset_id = ResourceTypeAndId {
            kind: refs_asset::RefsAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };
        let asset_content = create_ref_asset("test_content", ResourceId::new_explicit(9));
        let asset_checksum = content_store
            .write_content(&asset_content)
            .await
            .expect("to store asset");
        assert_eq!(
            content_store.read_content(&asset_checksum).await.unwrap(),
            asset_content
        );

        let binary_assetfile = write_assetfile(
            std::iter::once((asset_id, asset_checksum)),
            std::iter::empty(),
            &content_store,
        )
        .await
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

    #[tokio::test]
    async fn two_dependent_assets() {
        let content_store = Arc::new(Box::new(MemoryProvider::new()));

        let child_id = ResourceTypeAndId {
            kind: refs_asset::RefsAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };
        let child_content = create_ref_asset("child", ResourceId::new_explicit(9));
        let child_checksum = content_store
            .write_content(&child_content)
            .await
            .expect("to store asset");
        assert_eq!(
            content_store.read_content(&child_checksum).await.unwrap(),
            child_content
        );

        let parent_id = ResourceTypeAndId {
            kind: refs_asset::RefsAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };
        let parent_content = create_ref_asset("parent", ResourceId::new_explicit(1));
        let parent_checksum = content_store
            .write_content(&parent_content)
            .await
            .expect("to store asset");
        assert_eq!(
            content_store.read_content(&parent_checksum).await.unwrap(),
            parent_content
        );

        let reference_list = vec![(parent_id, (child_id, child_id))];

        let parent_assetfile = write_assetfile(
            std::iter::once((parent_id, parent_checksum)),
            reference_list.iter().copied(),
            &content_store,
        )
        .await
        .expect("asset file");

        let _child_assetfile = write_assetfile(
            std::iter::once((child_id, child_checksum)),
            std::iter::empty(),
            &content_store,
        )
        .await
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
