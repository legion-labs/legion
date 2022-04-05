//! Module containing information about compiled assets.

use std::sync::Arc;

use lgn_content_store2::Identifier;
use serde::{Deserialize, Serialize};

use crate::ResourceTypeAndId;

/// Description of a compiled asset.
#[derive(Serialize, Deserialize)]
pub struct CompiledAsset {
    /// The id of the asset.
    pub resource_id: ResourceTypeAndId,
    /// The checksum of the asset.
    pub content_id: Identifier,
}

/// `Manifest` contains storage information about assets - their checksums and
/// sizes.
///
/// It can be safely shared between threads.
#[derive(Debug, Default, Clone)]
pub struct Manifest(Arc<flurry::HashMap<ResourceTypeAndId, Identifier>>);

impl Manifest {
    /// Retrieve information about `Asset` identified by a given
    /// [`crate::ResourceId`], if available.
    pub fn find(&self, type_id: ResourceTypeAndId) -> Option<Identifier> {
        self.0.pin().get(&type_id).cloned()
    }

    /// Add new information about an `Asset`.
    pub fn insert(&self, type_id: ResourceTypeAndId, checksum: Identifier) {
        self.0.pin().insert(type_id, checksum);
    }

    /// An iterator visiting all assets in manifest, in an arbitrary order.
    pub fn resources(&self) -> Vec<ResourceTypeAndId> {
        self.0.pin().keys().copied().collect::<Vec<_>>()
    }

    /// Extends the manifest with the contents of another manifest.
    // Suppress the warning because flurry::HashMap doesn't provide methods taking owning `self`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn extend(&self, other: Self) {
        for (id, value) in &other.0.pin() {
            self.0.pin().insert(*id, value.clone());
        }
    }

    /// apply the changes to our manifest and only retain what's changed/new
    pub fn get_delta(&self, other: &Self) -> Vec<ResourceTypeAndId> {
        let guard = self.0.pin();
        let mut changed = other
            .0
            .pin()
            .iter()
            .filter_map(|(id, checksum)| {
                if let Some(old_checksum) = guard.get(id) {
                    if checksum != old_checksum {
                        Some(*id)
                    } else {
                        None
                    }
                } else {
                    Some(*id)
                }
            })
            .collect::<Vec<_>>();

        changed.sort();
        changed
    }
}

impl Serialize for Manifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<CompiledAsset> = Vec::new();
        for (guid, checksum) in &self.0.pin() {
            entries.push(CompiledAsset {
                resource_id: *guid,
                content_id: checksum.clone(),
            });
        }
        entries.sort_by(|a, b| a.resource_id.partial_cmp(&b.resource_id).unwrap());
        entries.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Manifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let entries = Vec::<CompiledAsset>::deserialize(deserializer)?;
        let manifest = Self::default();
        for asset in entries {
            manifest.0.pin().insert(asset.resource_id, asset.content_id);
        }
        Ok(manifest)
    }
}
