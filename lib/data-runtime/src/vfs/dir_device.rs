use std::path::{Path, PathBuf};

use super::Device;
use crate::{to_string, ResourceId, ResourceType};

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
    fn load(&self, type_id: (ResourceType, ResourceId)) -> Option<Vec<u8>> {
        let path = self.dir.join(to_string(type_id));
        std::fs::read(path).ok()
    }
}
