//! Compiler utilities - transformations helpful when compiling data.

use std::convert::TryFrom;

use legion_data_offline::asset::AssetPathId;
use legion_data_runtime::AssetId;

/// Converts `AssetPathId` to `AssetId`.
pub fn pathid_to_assetid(path: &Option<AssetPathId>) -> Option<AssetId> {
    match path {
        Some(path) => AssetId::try_from(path.content_id()).ok(),
        None => None,
    }
}

/// Converts `AssetId` to underlying binary representation.
pub fn assetid_to_bin(id: Option<AssetId>) -> u128 {
    unsafe { std::mem::transmute::<Option<AssetId>, u128>(id) }
}

/// Converts `AssetPathId` through `AssetId` to binary representation.
pub fn pathid_to_binary(path: &Option<AssetPathId>) -> u128 {
    let id = pathid_to_assetid(path);
    assetid_to_bin(id)
}
