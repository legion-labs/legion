//! Module containing information about compiled assets.

use crate::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Description of a compiled asset.
#[derive(Serialize, Deserialize)]
pub struct CompiledAsset {
    /// The id of the asset.
    pub guid: AssetId,
    /// The checksum of the asset.
    pub checksum: i128,
    /// The size of the asset.
    pub size: usize,
}

/// `Manifest` contains storage information about assets - their checksums and sizes.
#[derive(Debug, Default)]
pub struct Manifest(HashMap<AssetId, (i128, usize)>);

impl Manifest {
    /// Retrieve information about `Asset` identified by a given [`AssetId`], if available.
    pub fn find(&self, id: AssetId) -> Option<(i128, usize)> {
        self.0.get(&id).cloned()
    }

    /// Add new information about an `Asset`.
    pub fn insert(&mut self, id: AssetId, checksum: i128, size: usize) {
        self.0.insert(id, (checksum, size));
    }
}

impl Serialize for Manifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<CompiledAsset> = Vec::new();
        for (guid, (checksum, size)) in self.0.iter() {
            entries.push(CompiledAsset {
                guid: *guid,
                checksum: *checksum,
                size: *size,
            });
        }
        entries.sort_by(|a, b| a.guid.partial_cmp(&b.guid).unwrap());
        entries.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Manifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let entries = Vec::<CompiledAsset>::deserialize(deserializer)?;
        let mut manifest = Self::default();
        for asset in entries {
            manifest.0.insert(asset.guid, (asset.checksum, asset.size));
        }
        Ok(manifest)
    }
}
