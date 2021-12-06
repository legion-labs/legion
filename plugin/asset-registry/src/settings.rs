use std::path::{Path, PathBuf};

use lgn_data_runtime::{ResourceId, ResourceType};
use lgn_utils::Settings;

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
    pub(crate) assets_to_load: Vec<(ResourceType, ResourceId)>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<(ResourceType, ResourceId)>,
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

impl Default for AssetRegistrySettings {
    fn default() -> Self {
        let settings = Settings::new();
        let project_folder = settings
            .get_absolute_path("editor_srv.project_dir")
            .unwrap_or_else(|| PathBuf::from("test/sample-data"));

        let content_store_path = project_folder.join("temp");
        let databuild_settings = {
            let build_bin = {
                std::env::current_exe().ok().map_or_else(
                    || panic!("cannot find test directory"),
                    |mut path| {
                        path.pop();
                        path.as_path().join("data-build.exe")
                    },
                )
            };
            let buildindex = content_store_path.clone();

            Some(DataBuildSettings::new(build_bin, buildindex))
        };
        Self {
            content_store_addr: content_store_path,
            game_manifest: project_folder.join("runtime").join("game.manifest"),
            assets_to_load: vec![],
            databuild_settings,
        }
    }
}
