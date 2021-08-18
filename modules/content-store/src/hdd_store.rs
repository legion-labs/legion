use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::{ContentStore, ContentStoreAddr};

/// Disk-based [`ContentStore`] implementation.
///
/// All assets are assumed to be stored directly in a directory provided to [`HddContentStore::open`].
pub struct HddContentStore {
    address: ContentStoreAddr,
}

impl HddContentStore {
    /// Opens [`HddContentStore`] in a given directory.
    pub fn open(root_path: ContentStoreAddr) -> Option<Self> {
        if !root_path.0.is_dir() {
            return None;
        }
        Some(Self { address: root_path })
    }

    fn asset_path(&self, id: i128) -> PathBuf {
        self.address.0.clone().join(id.to_string())
    }

    /// Address of the [`HddContentStore`]
    pub fn address(&self) -> ContentStoreAddr {
        self.address.clone()
    }
}

impl ContentStore for HddContentStore {
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
