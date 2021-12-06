//! Module containing information about compiled assets.

use std::collections::HashMap;

use lgn_content_store::Checksum;
use serde::{Deserialize, Serialize};

use crate::{ResourceId, ResourceType};

/// Description of a compiled asset.
#[derive(Serialize, Deserialize)]
pub struct CompiledAsset {
    /// The id of the asset.
    pub guid: (ResourceType, ResourceId),
    /// The checksum of the asset.
    pub checksum: Checksum,
    /// The size of the asset.
    pub size: usize,
}

/// `Manifest` contains storage information about assets - their checksums and sizes.
#[derive(Debug, Default)]
pub struct Manifest(HashMap<(ResourceType, ResourceId), (Checksum, usize)>);

impl Manifest {
    /// Retrieve information about `Asset` identified by a given [`ResourceId`], if available.
    pub fn find(&self, type_id: (ResourceType, ResourceId)) -> Option<(Checksum, usize)> {
        self.0.get(&type_id).copied()
    }

    /// Add new information about an `Asset`.
    pub fn insert(&mut self, type_id: (ResourceType, ResourceId), checksum: Checksum, size: usize) {
        self.0.insert(type_id, (checksum, size));
    }

    /// An iterator visiting all assets in manifest, in an arbitrary order.
    pub fn resources(&self) -> impl Iterator<Item = &(ResourceType, ResourceId)> {
        self.0.keys()
    }

    /// Extends the manifest with the contents of another manifest.
    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl Serialize for Manifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<CompiledAsset> = Vec::new();
        for (guid, (checksum, size)) in &self.0 {
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
