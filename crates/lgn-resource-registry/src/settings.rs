use std::path::{Path, PathBuf};

use lgn_content_store::ContentStoreAddr;

pub struct ResourceRegistrySettings {
    pub(crate) root_folder: PathBuf,
    pub(crate) source_control_path: String,
    pub(crate) build_output_db_addr: String,
    pub(crate) content_store_addr: ContentStoreAddr,
}

impl ResourceRegistrySettings {
    pub fn new(
        root_folder: impl AsRef<Path>,
        source_control_path: String,
        build_output_db_addr: String,
        content_store_addr: ContentStoreAddr,
    ) -> Self {
        Self {
            root_folder: root_folder.as_ref().to_owned(),
            source_control_path,
            build_output_db_addr,
            content_store_addr,
        }
    }

    pub fn root_folder(&self) -> &Path {
        self.root_folder.as_path()
    }
}
