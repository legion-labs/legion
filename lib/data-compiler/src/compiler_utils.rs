//! Compiler utilities - transformations helpful when compiling data.

use std::hash::{Hash, Hasher};

use bincode;
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::{ResourceId, ResourceType};
use lgn_utils::DefaultHasher;

use crate::{CompilerHash, Locale, Platform, Target};

/// Converts `ResourcePathId` to `ResourceId`.
pub fn path_id_to_asset_id(path: &Option<ResourcePathId>) -> Option<(ResourceType, ResourceId)> {
    path.as_ref().map(ResourcePathId::resource_id)
}

/// Converts `ResourcePathId` through `ResourceId` to binary representation.
pub fn path_id_to_binary(path: &Option<ResourcePathId>) -> Vec<u8> {
    let id = path_id_to_asset_id(path).unwrap();
    bincode::serialize(&id).unwrap()
}

/// Compiler hasher function that hashes only code and data versions
pub fn hash_code_and_data(
    code: &'static str,
    data: &'static str,
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
) -> CompilerHash {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    CompilerHash(hasher.finish())
}
