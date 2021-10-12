use legion_data_runtime::ResourceId;
use std::path::{Path, PathBuf};

pub struct AssetRegistrySettings {
    pub(crate) content_store_addr: PathBuf,
    pub(crate) game_manifest: PathBuf,
    pub(crate) assets_to_load: Vec<ResourceId>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        assets_to_load: Vec<ResourceId>,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            assets_to_load,
        }
    }
}
