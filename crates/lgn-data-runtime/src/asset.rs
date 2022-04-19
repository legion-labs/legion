use std::{any::Any, io, sync::Arc};

use lgn_data_model::TypeReflection;

use crate::{AssetRegistry, Resource, ResourcePathId};

/// Trait describing the resource loadable at runtime.
pub trait Asset: Resource {
    /// Loader of the asset.
    type Loader: AssetLoader + Send + Sync + Default + 'static;
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

/// Error type for `ResourceProcessorError`
#[derive(thiserror::Error, Debug)]
pub enum ResourceProcessorError {
    /// IOError fallthrough
    #[error("ResourceProcessor IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// AssetLoaderError fallthrough
    #[error("ResourceProcessor load failed: '{0}'")]
    AssetLoaderError(#[from] AssetLoaderError),

    /// Resource Serialization Error
    #[error("ResourceProcessor failed to serialize: '{0}'")]
    ResourceSerializationFailed(&'static str, String),

    /// AssetLoaderError fallthrough
    #[error("ResourceProcessor Reflection Error '{0}'")]
    ReflectionError(#[from] lgn_data_model::ReflectionError),
}

/// The trait defines a resource that can be stored in a [`Project`].
pub trait OfflineResource: Asset {
    /// Offline resource processor bound to the resource.
    type Processor: ResourceProcessor + Send + Sync + Default + 'static;
}

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating
    /// a new resource.
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync>;

    /// Interface returning a list of resources that `resource` depends on for
    /// building.
    fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId>;

    /// Return the name of the Resource type that the processor can process.
    fn get_resource_type_name(&self) -> Option<&'static str> {
        None
    }

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn io::Write,
    ) -> Result<usize, ResourceProcessorError>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError>;

    /// Interface to retrieve the Resource reflection interface
    fn get_resource_reflection<'a>(
        &self,
        _resource: &'a dyn Any,
    ) -> Option<&'a dyn TypeReflection> {
        None
    }

    /// Interface to retrieve the Resource reflection interface
    fn get_resource_reflection_mut<'a>(
        &self,
        _resource: &'a mut dyn Any,
    ) -> Option<&'a mut dyn TypeReflection> {
        None
    }
}
