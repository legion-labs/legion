//! Compiler utilities - transformations helpful when compiling data.

use legion_data_offline::ResourcePathId;
use legion_data_runtime::{ResourceId, ResourceType};

/// Converts `ResourcePathId` to `ResourceId`.
pub fn path_id_to_asset_id(
    path: &Option<ResourcePathId>,
    asset_type: ResourceType,
) -> Option<ResourceId> {
    path.as_ref().map(|p| p.push(asset_type).content_id())
}

/// Converts `ResourceId` to underlying binary representation.
pub fn asset_id_to_bin(id: Option<ResourceId>) -> u128 {
    unsafe { std::mem::transmute::<Option<ResourceId>, u128>(id) }
}

/// Converts `ResourcePathId` through `ResourceId` to binary representation.
pub fn path_id_to_binary(path: &Option<ResourcePathId>, asset_type: ResourceType) -> u128 {
    let id = path_id_to_asset_id(path, asset_type);
    asset_id_to_bin(id)
}
