use std::path::{Path, PathBuf};

use legion_content_store::ContentStore;

use crate::{manifest::Manifest, ResourceId};

// todo: this should return `Box<dyn io::Read>` instead of `Vec<u8>`.
pub(crate) trait Device: Send {
    fn lookup(&self, id: ResourceId) -> Option<Vec<u8>>;
}

/// Content addressable storage device. Resources are accessed through a manifest access table.
pub(crate) struct CasDevice {
    manifest: Manifest,
    content_store: Box<dyn ContentStore>,
}

impl CasDevice {
    pub(crate) fn new(manifest: Manifest, content_store: Box<dyn ContentStore>) -> Self {
        Self {
            manifest,
            content_store,
        }
    }
}

impl Device for CasDevice {
    fn lookup(&self, id: ResourceId) -> Option<Vec<u8>> {
        let (checksum, size) = self.manifest.find(id)?;
        let content = self.content_store.read(checksum.get())?;
        assert_eq!(content.len(), size);
        Some(content)
    }
}

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
    fn lookup(&self, id: ResourceId) -> Option<Vec<u8>> {
        let path = self.dir.join(format!("{:x}", id));
        std::fs::read(path).ok()
    }
}
