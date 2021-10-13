use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::{Checksum, ContentStore, ContentStoreAddr};

/// Disk-based [`ContentStore`] implementation.
///
/// All content is assumed to be stored directly in a directory provided to [`HddContentStore::open`].
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

    fn content_path(&self, id: Checksum) -> PathBuf {
        let bytes = id.to_be_bytes();
        let hex = hex::encode(bytes);
        self.address.0.clone().join(hex)
    }

    /// Address of the [`HddContentStore`]
    pub fn address(&self) -> ContentStoreAddr {
        self.address.clone()
    }
}

impl ContentStore for HddContentStore {
    fn write(&mut self, id: Checksum, data: &[u8]) -> Option<()> {
        let path = self.content_path(id);

        if path.exists() {
            Some(())
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .ok()?;

            file.write_all(data).ok()
        }
    }

    fn read(&self, id: Checksum) -> Option<Vec<u8>> {
        let path = self.content_path(id);
        fs::read(path).ok()
    }

    fn remove(&mut self, id: Checksum) {
        let path = self.content_path(id);
        let _res = fs::remove_file(path);
    }
}
