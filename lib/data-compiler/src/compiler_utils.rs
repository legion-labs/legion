//! Compiler utilities - transformations helpful when compiling data.

use std::convert::TryFrom;

use legion_data_offline::asset::AssetPathId;
use legion_data_runtime::AssetId;

/// Converts `AssetPathId` to `AssetId`.
pub fn path_id_to_asset_id(path: &Option<AssetPathId>) -> Option<AssetId> {
    path.as_ref()
        .map(|p| AssetId::try_from(p.content_id()).ok())?
}

/// Converts `AssetId` to underlying binary representation.
pub fn asset_id_to_bin(id: Option<AssetId>) -> u128 {
    unsafe { std::mem::transmute::<Option<AssetId>, u128>(id) }
}

/// Converts `AssetPathId` through `AssetId` to binary representation.
pub fn path_id_to_binary(path: &Option<AssetPathId>) -> u128 {
    let id = path_id_to_asset_id(path);
    asset_id_to_bin(id)
}
