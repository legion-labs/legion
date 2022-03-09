use std::path::{Path, PathBuf};

pub struct ResourceRegistrySettings {
    pub(crate) root_folder: PathBuf,
    pub(crate) source_control_path: String,
}

impl ResourceRegistrySettings {
    pub fn new(root_folder: impl AsRef<Path>, source_control_path: String) -> Self {
        Self {
            root_folder: root_folder.as_ref().to_owned(),
            source_control_path,
        }
    }

    pub fn root_folder(&self) -> &Path {
        self.root_folder.as_path()
    }
}
