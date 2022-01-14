use std::path::{Path, PathBuf};

use super::Device;
use crate::ResourceTypeAndId;

/// Directory storage device. Resources are stored if files named by their ids.
pub(crate) struct DirDevice {
    dir: PathBuf,
}

impl DirDevice {
    pub(crate) fn new(path: impl AsRef<Path>) -> Self {
        Self {
            dir: path.as_ref().to_owned(),
        }
    }
}

impl Device for DirDevice {
    fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        let path = self.dir.join(type_id.id.resource_path());
        std::fs::read(path).ok()
    }
}
