use crate::CompiledAsset;
use legion_assets::AssetId;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// A content-addressable storage interface for dealing with compiled assets.
// todo: change Option to Error
pub trait CompiledAssetStore {
    /// Write asset to the backing storage.
    fn write(&mut self, id: AssetId, data: &[u8]) -> Option<()>;

    /// Read asset from the backing storage.
    fn read(&self, id: AssetId) -> Option<Vec<u8>>;

    /// Remove asset from the backing storage.
    fn remove(&mut self, id: AssetId);

    /// Returns the description of the asset if it exists.
    ///
    /// This default implementation is quite inefficient as it involves reading the asset's
    /// content to calculate its md5.
    fn find(&self, id: AssetId) -> Option<CompiledAsset> {
        let data = self.read(id)?;

        Some(CompiledAsset::new(id, &data))
    }

    /// Stores the asset and validates its validity afterwards.
    ///
    /// This method calls [`write`](#method.write) to store the asset and [`read`](#method.read) afterwards
    /// to perform the validation.
    fn store(&mut self, id: AssetId, data: &[u8]) -> Option<CompiledAsset> {
        self.write(id, data)?;

        let asset = CompiledAsset::new(id, data);

        let read = self.read(id)?;

        if asset != CompiledAsset::new(id, &read) {
            self.remove(id);
            return None;
        }

        Some(asset)
    }
}

/*pub struct InMemoryCompiledAssetStore {
    assets: HashMap<AssetId, Vec<u8>>,
}

impl InMemoryCompiledAssetStore {
    pub fn new() -> Self {
        Self {
            assets: HashMap::<AssetId, Vec<u8>>::new(),
        }
    }
}

impl CompiledAssetStore for InMemoryCompiledAssetStore {
    fn write(&mut self, id: AssetId, data: &[u8]) -> Option<()> {
        if self.assets.contains_key(&id) {
            return None;
        }

        self.assets.insert(id, data.to_owned());
        Some(())
    }

    fn read(&self, id: AssetId) -> Option<Vec<u8>> {
        self.assets.get(&id).cloned()
    }

    fn remove(&mut self, id: AssetId) {
        self.assets.remove(&id);
    }
}*/

pub(crate) struct LocalCompiledAssetStore {
    root_path: PathBuf,
}

impl LocalCompiledAssetStore {
    pub fn new(root_path: &Path) -> Option<Self> {
        if !root_path.is_dir() {
            return None;
        }
        Some(Self {
            root_path: root_path.to_owned(),
        })
    }

    fn asset_path(&self, id: AssetId) -> PathBuf {
        self.root_path.clone().join(id.to_string())
    }
}

impl CompiledAssetStore for LocalCompiledAssetStore {
    fn write(&mut self, id: AssetId, data: &[u8]) -> Option<()> {
        let asset_path = self.asset_path(id);

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(asset_path)
            .ok()?;

        file.write_all(data).ok()
    }

    fn read(&self, id: AssetId) -> Option<Vec<u8>> {
        let asset_path = self.asset_path(id);
        fs::read(asset_path).ok()
    }

    fn remove(&mut self, id: AssetId) {
        let asset_path = self.asset_path(id);
        let _res = fs::remove_file(asset_path);
    }
}
