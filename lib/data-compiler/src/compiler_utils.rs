//! Compiler utilities - transformations helpful when compiling data.

use legion_data_offline::asset::AssetPathId;
use legion_data_runtime::{AssetId, AssetType};

/// Converts `AssetPathId` to `AssetId`.
pub fn path_id_to_asset_id(path: &Option<AssetPathId>, asset_type: AssetType) -> Option<AssetId> {
    path.as_ref().map(|p| p.push(asset_type).content_id())
}

/// Converts `AssetId` to underlying binary representation.
pub fn asset_id_to_bin(id: Option<AssetId>) -> u128 {
    unsafe { std::mem::transmute::<Option<AssetId>, u128>(id) }
}

/// Converts `AssetPathId` through `AssetId` to binary representation.
pub fn path_id_to_binary(path: &Option<AssetPathId>, asset_type: AssetType) -> u128 {
    let id = path_id_to_asset_id(path, asset_type);
    asset_id_to_bin(id)
}
