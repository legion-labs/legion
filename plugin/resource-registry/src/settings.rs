use std::path::{Path, PathBuf};

pub struct ResourceRegistrySettings {
    pub(crate) root_folder: PathBuf,
}

impl ResourceRegistrySettings {
    pub fn new(root_folder: impl AsRef<Path>) -> Self {
        Self {
            root_folder: root_folder.as_ref().to_owned(),
        }
    }
}
