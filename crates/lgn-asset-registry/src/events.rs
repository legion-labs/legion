use lgn_content_store2::ChunkIdentifier;
use lgn_data_runtime::ResourceTypeAndId;

pub struct LoadManifestEvent {
    pub manifest_id: ChunkIdentifier,
}

pub struct LoadAssetEvent {
    pub asset_id: ResourceTypeAndId,
}
