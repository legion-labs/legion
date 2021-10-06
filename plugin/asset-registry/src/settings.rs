use std::path::{Path, PathBuf};

pub struct AssetRegistrySettings {
    pub(crate) content_store_addr: PathBuf,
    pub(crate) game_manifest: PathBuf,
    pub(crate) root_asset: Option<String>,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        root_asset: Option<&str>,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            root_asset: root_asset.map(|s| s.to_owned()),
        }
    }
}
