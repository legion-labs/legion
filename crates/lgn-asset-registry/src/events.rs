use lgn_content_store::Identifier;
use lgn_data_runtime::ResourceTypeAndId;

pub enum AssetRegistryRequest {
    LoadManifest(Identifier),
    LoadAsset(ResourceTypeAndId),
}
