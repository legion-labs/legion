//! Module containing information about compiled assets.

use std::collections::HashMap;

use crate::AssetId;

/// Description of a compiled asset.
pub struct CompiledAsset {
    /// The id of the asset.
    pub guid: AssetId,
    /// The checksum of the asset.
    pub checksum: i128,
    /// The size of the asset.
    pub size: usize,
}

/// `Manifest` contains storage information about assets - their checksums and sizes.
#[derive(Debug)]
pub struct Manifest(HashMap<AssetId, (i128, usize)>);

impl Manifest {
    /// Create a new `Manifest`.
    pub fn default() -> Self {
        Self(HashMap::new())
    }

    /// Retrieve information about `Asset` identified by a given [`AssetId`], if available.
    pub fn find(&self, id: AssetId) -> Option<(i128, usize)> {
        self.0.get(&id).cloned()
    }

    /// Add new information about an `Asset`.
    pub fn insert(&mut self, id: AssetId, checksum: i128, size: usize) {
        self.0.insert(id, (checksum, size));
    }
}
