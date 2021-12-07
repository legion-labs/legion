//! Compiler utilities - transformations helpful when compiling data.

use std::hash::{Hash, Hasher};

use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::ResourceId;
use lgn_utils::DefaultHasher;

use crate::{compiler_api::CompilationEnv, CompilerHash};

/// Converts `ResourcePathId` to `ResourceId`.
pub fn path_id_to_asset_id(path: &Option<ResourcePathId>) -> Option<ResourceId> {
    path.as_ref().map(ResourcePathId::resource_id)
}

/// Converts `ResourceId` to underlying binary representation.
pub fn asset_id_to_bin(id: Option<ResourceId>) -> u128 {
    unsafe { std::mem::transmute::<Option<ResourceId>, u128>(id) }
}

/// Converts `ResourcePathId` through `ResourceId` to binary representation.
pub fn path_id_to_binary(path: &Option<ResourcePathId>) -> u128 {
    let id = path_id_to_asset_id(path);
    asset_id_to_bin(id)
}

/// Compiler hasher function that hashes only code and data versions
pub fn hash_code_and_data(
    code: &'static str,
    data: &'static str,
    _env: &CompilationEnv,
) -> CompilerHash {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    CompilerHash(hasher.finish())
}
