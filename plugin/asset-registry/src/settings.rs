use std::path::{Path, PathBuf};

use legion_data_runtime::ResourceId;

pub struct DataBuildSettings {
    pub(crate) build_bin: PathBuf,
    pub(crate) buildindex: PathBuf,
}

impl DataBuildSettings {
    pub fn new(build_bin: impl AsRef<Path>, buildindex: impl AsRef<Path>) -> Self {
        Self {
            build_bin: build_bin.as_ref().to_path_buf(),
            buildindex: buildindex.as_ref().to_path_buf(),
        }
    }
}

pub struct AssetRegistrySettings {
    pub(crate) content_store_addr: PathBuf,
    pub(crate) game_manifest: PathBuf,
    pub(crate) databuild_settings: Option<DataBuildSettings>,
    pub(crate) assets_to_load: Vec<ResourceId>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceId>,
        databuild_settings: Option<DataBuildSettings>,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            assets_to_load,
            databuild_settings,
        }
    }
}
