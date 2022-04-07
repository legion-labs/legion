use lgn_content_store::ChunkIdentifier;
use lgn_data_runtime::ResourceTypeAndId;

pub enum AssetRegistryRequest {
    LoadManifest(ChunkIdentifier),
    LoadAsset(ResourceTypeAndId),
}
