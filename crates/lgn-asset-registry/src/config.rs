use std::path::{Path, PathBuf};

use lgn_data_runtime::ResourceTypeAndId;

pub struct AssetRegistrySettings {
    pub(crate) game_manifest: Option<PathBuf>,
    pub(crate) assets_to_load: Vec<ResourceTypeAndId>,
}

impl AssetRegistrySettings {
    pub fn new(
        game_manifest: Option<impl AsRef<Path>>,
        assets_to_load: Vec<ResourceTypeAndId>,
    ) -> Self {
        Self {
            game_manifest: game_manifest.map(|path| path.as_ref().to_owned()),
            assets_to_load,
        }
    }
}

impl Default for AssetRegistrySettings {
    fn default() -> Self {
        let project_folder =
            lgn_config::get_or("editor_srv.project_dir", PathBuf::from("tests/sample-data"))
                .unwrap();

        Self {
            game_manifest: Some(project_folder.join("runtime").join("game.manifest")),
            assets_to_load: vec![],
        }
    }
}
