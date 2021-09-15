//! Module containing information about compiled assets.

use crate::AssetId;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};

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

    /// Construct `Manifest` by reading in persisted information
    pub fn import(file: &File) -> Self {
        let mut manifest = Self::default();
        let compiled_assets: serde_json::Result<Vec<CompiledAsset>> = serde_json::from_reader(file);
        if let Ok(compiled_assets) = compiled_assets {
            for compiled_asset in compiled_assets {
                manifest.insert(
                    compiled_asset.guid,
                    compiled_asset.checksum,
                    compiled_asset.size,
                );
            }
        }
        manifest
    }

    /// Export to list of compiled assets
    pub fn export(&self, file: &File) {
        let mut compiled_assets = Vec::new();
        for (id, info) in &self.0 {
            compiled_assets.push(CompiledAsset {
                guid: *id,
                checksum: info.0,
                size: info.1,
            });
        }
        serde_json::to_writer_pretty(file, &compiled_assets).unwrap();
    }
}
