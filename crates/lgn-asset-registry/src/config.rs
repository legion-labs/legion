use std::path::{Path, PathBuf};

use lgn_config::Config;
use lgn_data_runtime::ResourceTypeAndId;

pub struct DataBuildConfig {
    pub(crate) build_bin: PathBuf,
    pub(crate) buildindex: PathBuf,
}

impl DataBuildConfig {
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
    pub(crate) databuild_config: Option<DataBuildConfig>,
    pub(crate) assets_to_load: Vec<ResourceTypeAndId>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceTypeAndId>,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            assets_to_load,
            databuild_config: None,
        }
    }

    /// Create config that support rebuilding resources upon reload.
    /// Build index is assumed to be under the `content_store_addr` location.
    pub fn new_with_rebuild(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceTypeAndId>,
        build_bin: impl AsRef<Path>,
    ) -> Self {
        let content_store_addr = content_store_addr.as_ref().to_owned();
        let databuild_config = {
            let buildindex = &content_store_addr;
            Some(DataBuildConfig::new(build_bin, buildindex))
        };

        Self {
            content_store_addr,
            game_manifest: game_manifest.as_ref().to_owned(),
            assets_to_load,
            databuild_config,
        }
    }
}

impl Default for AssetRegistrySettings {
    fn default() -> Self {
        let config = Config::new();
        let project_folder = config
            .get_absolute_path("editor_srv.project_dir")
            .unwrap_or_else(|| PathBuf::from("tests/sample-data"));

        let content_store_path = project_folder.join("temp");

        Self {
            content_store_addr: content_store_path,
            game_manifest: project_folder.join("runtime").join("game.manifest"),
            assets_to_load: vec![],
            databuild_config: None,
        }
    }
}
