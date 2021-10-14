use std::{
    any::Any,
    io,
    sync::{Arc, Mutex},
};

use crate::{AssetRegistry, Resource};

/// Trait describing the resource loadable at runtime.
pub trait Asset: Resource {
    /// Loader of the asset.
    type Loader: AssetLoader + Send + Default + 'static;
}

/// An interface allowing to create and initialize assets.
pub trait AssetLoader {
    /// Asset loading interface.
    ///
    /// # Errors
    ///
    /// Will return 'Err' if unable to deserialize asset
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>>;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync));

    /// An asset loader can keep a reference to the asset registry, for use in asset initialization
    fn register_registry(&mut self, _registry: Arc<Mutex<AssetRegistry>>) {}
}
