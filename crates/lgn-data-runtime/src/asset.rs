use std::{any::Any, io, sync::Arc};

use crate::{AssetRegistry, Resource};

/// Trait describing the resource loadable at runtime.
pub trait Asset: Resource {
    /// Loader of the asset.
    type Loader: AssetLoader + Send + Default + 'static;
}

/// Error for `AssetLoader` implementation
#[derive(thiserror::Error, Debug)]
pub enum AssetLoaderError {
    /// Failed to load a resource
    #[error("AssetLoader '{0}' ({1})")]
    ErrorLoading(&'static str, String),

    /// IOError fallback
    #[error("AssetLoader IOError: {0}")]
    IOError(#[from] std::io::Error),

    /// IOError fallback
    #[error("AssetLoader Invalid Uft8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// An interface allowing to create and initialize assets.
pub trait AssetLoader {
    /// Asset loading interface.
    ///
    /// # Errors
    ///
    /// Will return 'Err' if unable to deserialize asset
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError>;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync));

    /// An asset loader can keep a reference to the asset registry, for use in
    /// asset initialization
    fn register_registry(&mut self, _registry: Arc<AssetRegistry>) {}
}
