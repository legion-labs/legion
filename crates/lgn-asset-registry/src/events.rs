use lgn_content_store::indexing::TreeIdentifier;
use lgn_data_runtime::ResourceTypeAndId;

pub enum AssetRegistryRequest {
    LoadManifest(TreeIdentifier),
    LoadAsset(ResourceTypeAndId),
}
