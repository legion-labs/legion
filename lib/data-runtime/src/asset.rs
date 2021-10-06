use std::{any::Any, io};

use crate::Resource;

/// Trait describing the resource loadable at runtime.
pub trait Asset: Resource {
    /// Loader of the asset.
    type Loader: AssetLoader + Send + Default + 'static;
}

/// An interface allowing to create and initialize assets.
pub trait AssetLoader {
    /// Asset loading interface.
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>>;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync));
}
