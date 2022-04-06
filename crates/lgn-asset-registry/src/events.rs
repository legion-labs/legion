use lgn_content_store2::ChunkIdentifier;
use lgn_data_runtime::ResourceTypeAndId;

pub enum AssetRegistryRequest {
    LoadManifest(ChunkIdentifier),
    LoadAsset(ResourceTypeAndId),
}
