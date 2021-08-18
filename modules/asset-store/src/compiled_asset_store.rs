//! [`CompiledAssetStore`] is an interface used to stor the results of data compilation.
//!
//! [`CompiledAssetStore`] functions as a *content-addressable storage* - using the [`compute_asset_checksum`]
//! function to calculate the checksum of stored content.
//!
//! Currently the only [`LocalCompiledAssetStore`] is available which provides disk-based implementation of [`CompiledAssetStore`].

use core::fmt;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::{self, OpenOptions},
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
};

/// Returns the hash of the provided data.
pub fn compute_asset_checksum(data: &[u8]) -> i128 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as i128
}

/// The address of the `Compiled Asset Store`.
///
/// For now, it is equivalent to a `PathBuf` since there is only support for on-disk `LocalCompiledAssetStore`.
/// In the future the address could be representing a remote machine or service.
#[derive(Clone, Debug)]
pub struct CompiledAssetStoreAddr(PathBuf);

impl From<&str> for CompiledAssetStoreAddr {
    fn from(path: &str) -> Self {
        Self(PathBuf::from(path))
    }
}

impl From<PathBuf> for CompiledAssetStoreAddr {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&Path> for CompiledAssetStoreAddr {
    fn from(path: &Path) -> Self {
        Self(path.to_owned())
    }
}

impl fmt::Display for CompiledAssetStoreAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0.display()))
    }
}

/// A content-addressable storage interface for dealing with compiled assets.
// todo: change Option to Error
pub trait CompiledAssetStore: Send {
    /// Write asset to the backing storage.
    fn write(&mut self, id: i128, data: &[u8]) -> Option<()>;

    /// Read asset from the backing storage.
    fn read(&self, id: i128) -> Option<Vec<u8>>;

    /// Remove asset from the backing storage.
    fn remove(&mut self, id: i128);

    /// Returns the description of the asset if it exists.
    ///
    /// This default implementation is quite inefficient as it involves reading the asset's
    /// content to calculate its checksum.
    fn exists(&self, id: i128) -> bool {
        self.read(id).is_some()
    }

    /// Stores the asset and validates its validity afterwards.
    ///
    /// This method calls [`write`](#method.write) to store the asset and [`read`](#method.read) afterwards
    /// to perform the validation.
    fn store(&mut self, data: &[u8]) -> Option<i128> {
        let id = compute_asset_checksum(data);
        self.write(id, data)?;

        let read = self.read(id)?;

        if id != compute_asset_checksum(&read) {
            self.remove(id);
            return None;
        }

        Some(id)
    }
}

/// In-memory `CompiledAssetStore` implementation.
///
/// Handy implementation for testing purposes.
pub struct InMemoryCompiledAssetStore {
    assets: HashMap<i128, Vec<u8>>,
}

impl InMemoryCompiledAssetStore {
    /// Create empty in-memory `CompiledAssetStore`.
    pub fn default() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}

impl CompiledAssetStore for InMemoryCompiledAssetStore {
    fn write(&mut self, id: i128, data: &[u8]) -> Option<()> {
        if self.assets.contains_key(&id) {
            return None;
        }

        self.assets.insert(id, data.to_owned());
        Some(())
    }

    fn read(&self, id: i128) -> Option<Vec<u8>> {
        self.assets.get(&id).cloned()
    }

    fn remove(&mut self, id: i128) {
        self.assets.remove(&id);
    }
}

/// Disk-based `CompiledAssetStore` implementation.
///
/// All assets are assumed to be stored directly in a given root directory.
pub struct LocalCompiledAssetStore {
    address: CompiledAssetStoreAddr,
}

impl LocalCompiledAssetStore {
    /// Opens [`LocalCompiledAssetStore`] in a given directory.
    pub fn open(root_path: CompiledAssetStoreAddr) -> Option<Self> {
        if !root_path.0.is_dir() {
            return None;
        }
        Some(Self { address: root_path })
    }

    fn asset_path(&self, id: i128) -> PathBuf {
        self.address.0.clone().join(id.to_string())
    }

    /// Address of the [`LocalCompiledAssetStore`]
    pub fn address(&self) -> CompiledAssetStoreAddr {
        self.address.clone()
    }
}

impl CompiledAssetStore for LocalCompiledAssetStore {
    fn write(&mut self, id: i128, data: &[u8]) -> Option<()> {
        let asset_path = self.asset_path(id);

        if asset_path.exists() {
            Some(())
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(asset_path)
                .ok()?;

            file.write_all(data).ok()
        }
    }

    fn read(&self, id: i128) -> Option<Vec<u8>> {
        let asset_path = self.asset_path(id);
        fs::read(asset_path).ok()
    }

    fn remove(&mut self, id: i128) {
        let asset_path = self.asset_path(id);
        let _res = fs::remove_file(asset_path);
    }
}
