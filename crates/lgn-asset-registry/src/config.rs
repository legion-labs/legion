use std::path::{Path, PathBuf};

use lgn_content_store::ContentStoreAddr;
use lgn_data_runtime::ResourceTypeAndId;

pub struct DataBuildConfig {
    pub(crate) build_bin: PathBuf,
    pub(crate) output_db_addr: String,
    pub(crate) project: PathBuf,
}

impl DataBuildConfig {
    pub fn new(
        build_bin: impl AsRef<Path>,
        output_db_addr: String,
        project: impl AsRef<Path>,
    ) -> Self {
        Self {
            build_bin: build_bin.as_ref().to_path_buf(),
            output_db_addr,
            project: project.as_ref().to_path_buf(),
        }
    }
}

pub struct AssetRegistrySettings {
    pub(crate) content_store_addr: ContentStoreAddr,
    pub(crate) game_manifest: PathBuf,
    pub(crate) databuild_config: Option<DataBuildConfig>,
    pub(crate) assets_to_load: Vec<ResourceTypeAndId>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: ContentStoreAddr,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceTypeAndId>,
    ) -> Self {
        Self {
            content_store_addr,
            game_manifest: game_manifest.as_ref().to_owned(),
            assets_to_load,
            databuild_config: None,
        }
    }

    /// Create config that support rebuilding resources upon reload.
    /// Build index is assumed to be under the `content_store_addr` location.
    pub fn new_with_rebuild(
        content_store_addr: ContentStoreAddr,
        output_db_addr: String,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceTypeAndId>,
        project: impl AsRef<Path>,
        build_bin: impl AsRef<Path>,
    ) -> Self {
        let databuild_config = { Some(DataBuildConfig::new(build_bin, output_db_addr, project)) };

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
        let project_folder = lgn_config::get_absolute_path_or(
            "editor_srv.project_dir",
            PathBuf::from("tests/sample-data"),
        )
        .unwrap();

        let content_store_path = project_folder.join("temp");

        Self {
            content_store_addr: ContentStoreAddr::from(content_store_path.to_str().unwrap()),
            game_manifest: project_folder.join("runtime").join("game.manifest"),
            assets_to_load: vec![],
            databuild_config: None,
        }
    }
}
