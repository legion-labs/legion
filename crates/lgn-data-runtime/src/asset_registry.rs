use std::{
    collections::HashMap,
    path::Path,
    pin::Pin,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

use futures::{future::Shared, Future, FutureExt};

use lgn_content_store::Provider;
use lgn_ecs::schedule::SystemLabel;
use lgn_tracing::{debug, error};
use slotmap::SlotMap;

use crate::{
    asset_loader::AssetLoaderIO, manifest::Manifest, vfs, AssetRegistryHandleKey,
    ComponentInstaller, EditHandle, EditHandleUntyped, Handle, HandleEntry, HandleUntyped,
    Resource, ResourceDescriptor, ResourceId, ResourceInstaller, ResourcePathId, ResourceProcessor,
    ResourceType, ResourceTypeAndId,
};

/// Context about a load request
pub struct LoadRequest {
    /// Id of the primary resource getting loaded by this request
    pub primary_id: ResourceTypeAndId,
    /// Asset Registry where to register the loaded/installed resources
    pub asset_registry: Arc<AssetRegistry>,
}

impl LoadRequest {
    fn new(primary_id: ResourceTypeAndId, asset_registry: Arc<AssetRegistry>) -> Self {
        Self {
            primary_id,
            asset_registry,
        }
    }
}

/// Error type for Asset Registry
#[derive(thiserror::Error, Debug, Clone)]
pub enum AssetRegistryError {
    /// Error when a resource is not found
    #[error("Resource '{0:?}' was not found")]
    ResourceNotFound(ResourceTypeAndId),

    /// Error when a handle is invalid
    #[error("Invalid Resource Handle '{0:?}' was not found")]
    InvalidHandle(AssetRegistryHandleKey),

    /// Resource Serialization Error
    #[error("ResourceProcessor failed to serialize: '{0}'")]
    ResourceSerializationFailed(&'static str, String),

    /// General IO Error when loading a resource
    #[error("IO error: {0}")]
    IOError(std::sync::Arc<std::io::Error>),

    /// AssetLoader for a type not present
    #[error("ResourceInstaller for ResourceType '{0:?}' not found")]
    ResourceInstallerNotFound(ResourceType),

    /// Processor not found
    #[error("Processor '{0}'not found")]
    ProcessorNotFound(ResourceType),

    /// AssetLoaderError fallthrough
    #[error("ResourceProcessor Reflection Error '{0}'")]
    ReflectionError(#[from] lgn_data_model::ReflectionError),
}

impl From<std::io::Error> for AssetRegistryError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(std::sync::Arc::new(err))
    }
}

/// Return a Guarded Ref to a Asset
pub struct AssetRegistryGuard<'a, T: ?Sized + 'a> {
    _guard: RwLockReadGuard<'a, Inner>,
    ptr: *const T,
}

impl<'a, T: ?Sized + 'a> std::ops::Deref for AssetRegistryGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

/// Options which can be used to configure the creation of [`AssetRegistry`].
pub struct AssetRegistryOptions {
    processors: HashMap<ResourceType, Arc<dyn ResourceProcessor>>,
    resource_installers: HashMap<ResourceType, Arc<dyn ResourceInstaller>>,
    component_installers: HashMap<std::any::TypeId, Arc<dyn ComponentInstaller>>,

    devices: Vec<Box<dyn vfs::Device + Send>>,
}

impl AssetRegistryOptions {
    /// Creates a blank set of options for [`AssetRegistry`] configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
            resource_installers: HashMap::new(),
            component_installers: HashMap::new(),
            devices: vec![],
        }
    }

    /// Add a Processor for a specific `ResourceType`
    pub fn add_processor(
        &mut self,
        kind: ResourceType,
        proc: Arc<dyn ResourceProcessor>,
    ) -> &mut Self {
        self.processors.insert(kind, proc);
        self
    }

    /// Add a Resource Installer for a specific `ResourceType`
    pub fn add_resource_installer(
        &mut self,
        resource_type: ResourceType,
        installer: Arc<dyn ResourceInstaller>,
    ) -> &mut Self {
        self.resource_installers.insert(resource_type, installer);
        self
    }

    /// Add a default Resource Installer for a specific `ResourceType` (if none exists already)
    pub fn add_default_resource_installer(
        &mut self,
        resource_type: ResourceType,
        installer: Arc<dyn ResourceInstaller>,
    ) -> &mut Self {
        self.resource_installers
            .entry(resource_type)
            .or_insert(installer);
        self
    }

    /// Add a Component installer for a specific type(s)
    pub fn add_component_installer(
        &mut self,
        component_ids: &[std::any::TypeId],
        installer: Arc<dyn ComponentInstaller>,
    ) -> &mut Self {
        for component_id in component_ids.iter().skip(1) {
            self.component_installers
                .insert(*component_id, installer.clone());
        }
        component_ids
            .first()
            .and_then(|first_id| self.component_installers.insert(*first_id, installer));
        self
    }

    /// Specifying `directory device` will mount a device that allows to read
    /// resources from a specified directory.
    #[must_use]
    pub fn add_device_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.devices.push(Box::new(vfs::DirDevice::new(path)));
        self
    }

    /// Specifying `directory device` will mount a device that allows to read
    /// resources from a specified directory.
    pub fn add_device_dir_mut(&mut self, path: impl AsRef<Path>) -> &Self {
        self.devices.push(Box::new(vfs::DirDevice::new(path)));
        self
    }

    /// Specifying `content-addressable storage device` will mount a device that
    /// allows to read resources from a specified content store through
    /// provided manifest.
    #[must_use]
    pub fn add_device_cas(mut self, provider: Arc<Provider>, manifest: Manifest) -> Self {
        self.devices
            .push(Box::new(vfs::CasDevice::new(Some(manifest), provider)));
        self
    }

    /// Specifying `content-addressable storage device` will mount a device that
    /// allows to read resources from a specified content store.
    /// It must subsequently be provided with a manifest to be able to fetch resources.
    #[must_use]
    pub fn add_device_cas_with_delayed_manifest(mut self, provider: Arc<Provider>) -> Self {
        self.devices
            .push(Box::new(vfs::CasDevice::new(None, provider)));
        self
    }

    /// Specifying `build device` will mount a device that allows to build
    /// resources as they are being requested.
    ///
    /// `force_recompile` if set will cause each load request to go through data
    /// compilation.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn add_device_build(
        mut self,
        provider: Arc<Provider>,
        manifest: Manifest,
        build_bin: impl AsRef<Path>,
        output_db_addr: String,
        project: impl AsRef<Path>,
        force_recompile: bool,
    ) -> Self {
        self.devices.push(Box::new(vfs::BuildDevice::new(
            manifest,
            provider,
            build_bin,
            output_db_addr,
            project,
            force_recompile,
        )));
        self
    }

    /// Creates [`AssetRegistry`] based on `AssetRegistryOptions`.
    pub async fn create(self) -> Arc<AssetRegistry> {
        Arc::new(AssetRegistry {
            inner: RwLock::new(Inner {
                handles: SlotMap::with_key(),
                index: HashMap::new(),
            }),
            loader: AssetLoaderIO::new(self.devices, self.resource_installers),
            processors: self.processors,
            component_installers: self.component_installers,
            removal_queue: std::sync::Mutex::new(Vec::new()),
            pending_requests: std::sync::Mutex::new(HashMap::new()),
        })
    }
}

struct Inner {
    handles: SlotMap<AssetRegistryHandleKey, Arc<HandleEntry>>,
    index: HashMap<ResourceId, AssetRegistryHandleKey>,
}

/// Registry of all loaded [`Resource`]s.
///
/// Provides an API to load assets by their [`crate::ResourceId`]. The lifetime
/// of an [`Resource`] is determined by the reference counted [`HandleUntyped`]
/// and [`Handle`].
///
/// # Safety:
///
/// The `update` method can only be called when no outstanding references `Ref`
/// to resources exist. No other method can be called concurrently with `update`
/// method.
///
/// [`Handle`]: [`crate::Handle`]
pub struct AssetRegistry {
    inner: RwLock<Inner>,
    loader: AssetLoaderIO,
    processors: HashMap<ResourceType, Arc<dyn ResourceProcessor>>,
    component_installers: HashMap<std::any::TypeId, Arc<dyn ComponentInstaller>>,
    removal_queue: std::sync::Mutex<Vec<AssetRegistryHandleKey>>,
    pending_requests: std::sync::Mutex<HashMap<ResourceId, SharedLoadFuture>>,
}

type SharedLoadFuture =
    Shared<Pin<Box<dyn Future<Output = Result<HandleUntyped, AssetRegistryError>> + Send>>>;

/// Label to use for scheduling systems that require the `AssetRegistry`
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum AssetRegistryScheduling {
    /// AssetRegistry has been created
    AssetRegistryCreated,
}

/// Async reader type for `AssetRegistry`/`AssetLoader`
pub type AssetRegistryReader = Pin<Box<dyn tokio::io::AsyncRead + Send>>;

impl AssetRegistry {
    fn read_inner(&self) -> RwLockReadGuard<'_, Inner> {
        self.inner.read().unwrap()
    }

    fn write_inner(&self) -> RwLockWriteGuard<'_, Inner> {
        self.inner.write().unwrap()
    }

    /// Trigger a reload of a given primary resource.
    pub async fn reload(&self, resource_id: ResourceTypeAndId) {
        let future = self.new_load_request(resource_id);
        if let Err(err) = future.await {
            lgn_tracing::error!("Reload failed: {}", err);
        }
    }

    pub(crate) fn mark_for_cleanup(&self, key: AssetRegistryHandleKey) {
        self.removal_queue.lock().unwrap().push(key);
    }

    /// Interface to return an existing Handle for a `Resource_id` (won't trigger a load if it doesn't exists)
    pub fn lookup_untyped(&self, resource_id: &ResourceTypeAndId) -> Option<HandleUntyped> {
        let guard = self.inner.read().unwrap();
        if let Some(key) = guard.index.get(&resource_id.id).copied() {
            if let Some(entry) = guard.handles.get(key) {
                return Some(HandleUntyped::new(key, entry.clone()));
            }
        }
        None
    }

    /// Interface to return an existing Handle for a `Resource_id` (won't trigger a load if it doesn't exists)
    pub fn lookup<T: Resource>(&self, resource_id: &ResourceTypeAndId) -> Option<Handle<T>> {
        Some(self.lookup_untyped(resource_id)?.into())
    }

    fn arc_self(&self) -> Arc<Self> {
        let registry = unsafe { Arc::from_raw(self as *const Self) };
        let result = registry.clone();
        let _oldself = Arc::into_raw(registry);
        result
    }

    fn weak_self(&self) -> Weak<Self> {
        let registry = unsafe { Arc::from_raw(self as *const Self) };
        let result = Arc::downgrade(&registry);
        let _oldself = Arc::into_raw(registry);
        result
    }

    /// Register a new resource at a specific Id. Replace the existing resource with the same id
    /// # Errors
    /// Return `AssetRegistryError` on failure
    pub fn set_resource(
        &self,
        id: ResourceTypeAndId,
        new_resource: Box<dyn Resource>,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let (key, entry) = {
            let guard = self.inner.read().unwrap();
            let key = if let Some(key) = guard.index.get(&id.id).copied() {
                if let Some(entry) = guard.handles.get(key) {
                    let mut asset = entry.asset.write().unwrap();
                    let asset: &mut Box<dyn Resource> = &mut asset;
                    let _old = std::mem::replace::<Box<dyn Resource>>(asset, new_resource);
                    debug!("Replacing {:?} ({:?})", id, key);
                    (key, entry.clone())
                } else {
                    error!(
                        "AssetRegistry index out of sync, cannot find entry for {}",
                        id
                    );
                    return Err(AssetRegistryError::ResourceNotFound(id));
                }
            } else {
                drop(guard);
                let mut guard = self.inner.write().unwrap();
                let entry = HandleEntry::new(id, new_resource, self.weak_self());
                let key = guard.handles.insert(entry.clone());
                guard.index.insert(id.id, key);
                debug!("Registering {:?} ({:?})", id, key);
                (key, entry)
            };
            key
        };
        Ok(HandleUntyped::new(key, entry))
    }

    fn new_load_request(&self, id: ResourceTypeAndId) -> SharedLoadFuture {
        let registry = self.arc_self();
        let future = async move {
            let mut request = LoadRequest::new(id, registry.clone());
            let handle = registry.loader.load_from_device(id, &mut request).await?;
            Ok(handle)
        }
        .boxed()
        .shared();
        future
    }

    /// Load a Resource asynchronously and return an owning Handle to the data
    /// # Errors
    /// Return `AssetRegistryError` on failure
    pub async fn load_async_untyped(
        &self,
        id: ResourceTypeAndId,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        {
            let guard = self.inner.read().unwrap();
            if let Some(key) = guard.index.get(&id.id).copied() {
                if let Some(entry) = guard.handles.get(key) {
                    return Ok(HandleUntyped::new(key, entry.clone()));
                }
            }
        }

        let existing_future = self.pending_requests.lock().unwrap().get(&id.id).cloned();
        let future = if let Some(cloned_future) = existing_future {
            cloned_future
        } else {
            let future = self.new_load_request(id);
            self.pending_requests
                .lock()
                .unwrap()
                .insert(id.id, future.clone());
            future
        };

        let result = future.await;
        self.pending_requests.lock().unwrap().remove(&id.id);
        result
    }

    /// Load a Resource asynchronously and return a Handle<T> to the data
    /// # Errors
    /// Return `AssetRegistryError` on failure
    pub async fn load_async<T: Resource>(
        &self,
        id: ResourceTypeAndId,
    ) -> Result<Handle<T>, AssetRegistryError> {
        Ok(self.load_async_untyped(id).await?.into())
    }

    /// Create a Clone that can be edited and commited
    pub fn edit<T: Resource>(&self, handle: &Handle<T>) -> Option<EditHandle<T>> {
        let boxed_asset = self
            .inner
            .read()
            .unwrap()
            .handles
            .get(handle.key())
            .and_then(|entry| {
                if let Some(asset) = entry.asset.read().unwrap().downcast_ref::<T>() {
                    let edit = asset.clone_dyn();
                    let raw: *mut dyn Resource = Box::into_raw(edit);
                    let boxed_asset = unsafe { Box::from_raw(raw.cast::<T>()) };
                    Some(boxed_asset)
                } else {
                    None
                }
            })?;

        Some(EditHandle::<T>::new(handle.clone(), boxed_asset))
    }

    /// Create a Clone that can be edited and commited
    pub fn edit_untyped(&self, handle: &HandleUntyped) -> Option<EditHandleUntyped> {
        let boxed_asset = self
            .inner
            .read()
            .unwrap()
            .handles
            .get(handle.key())
            .map(|entry| {
                let asset = entry.asset.read().unwrap();
                let edit = asset.clone_dyn();
                let raw: *mut dyn Resource = Box::into_raw(edit);
                unsafe { Box::from_raw(raw as *mut dyn Resource) }
            })?;

        Some(EditHandleUntyped::new(handle.clone(), boxed_asset))
    }

    /// Commit a Resource
    pub fn commit<T: Resource>(&self, edit_handle: EditHandle<T>) -> Handle<T> {
        let mut guard = self.inner.write().unwrap();
        if let Some(entry) = guard.handles.get_mut(edit_handle.handle.key()) {
            let mut asset = entry.asset.write().unwrap();
            let asset: &mut Box<dyn Resource> = &mut asset;
            let _old_value = std::mem::replace::<Box<dyn Resource>>(asset, edit_handle.asset);
        }
        edit_handle.handle
    }

    /// Commit a Resource
    pub fn commit_untyped(&self, edit_handle: EditHandleUntyped) -> HandleUntyped {
        let mut guard = self.inner.write().unwrap();
        if let Some(entry) = guard.handles.get_mut(edit_handle.handle.key()) {
            let mut asset = entry.asset.write().unwrap();
            let asset: &mut Box<dyn Resource> = &mut asset;
            let _old_value = std::mem::replace::<Box<dyn Resource>>(asset, edit_handle.asset);
        }
        edit_handle.handle
    }

    pub(crate) fn collect_dropped_handles(&self) {
        let removals: Vec<AssetRegistryHandleKey> =
            std::mem::take(self.removal_queue.lock().unwrap().as_mut());

        if !removals.is_empty() {
            let mut guard = self.write_inner();
            for key in removals {
                if guard
                    .handles
                    .get(key)
                    .map(|entry| Arc::strong_count(entry) == 1)
                    .is_some()
                {
                    if let Some(entry) = guard.handles.remove(key) {
                        lgn_tracing::debug!("Dropping {:?}({:?})", entry.id, key,);
                        guard.index.remove(&entry.id.id);
                    }
                }
            }
        }
    }

    /// Get Component Installer
    pub fn get_component_installer(
        &self,
        type_id: std::any::TypeId,
    ) -> Option<Arc<dyn ComponentInstaller>> {
        self.component_installers.get(&type_id).cloned()
    }

    /// Unloads assets based on their reference counts.
    pub fn update(&self) {
        self.collect_dropped_handles();
    }

    /// Return a resource in a default state
    pub fn new_resource<T: Resource + ResourceDescriptor>(&self) -> Option<Handle<T>> {
        let id = ResourceTypeAndId {
            kind: T::TYPE,
            id: ResourceId::new(),
        };
        self.new_resource_with_id::<T>(id)
    }

    /// Return a resource in a default state with a specific Id
    pub fn new_resource_with_id<T: Resource>(&self, id: ResourceTypeAndId) -> Option<Handle<T>> {
        if let Some(processor) = self.processors.get(&id.kind) {
            let resource = processor.new_resource();

            let entry = HandleEntry::new(id, resource, self.weak_self());
            let mut guard = self.write_inner();
            let key = guard.handles.insert(entry.clone());
            guard.index.insert(id.id, key);
            Some(HandleUntyped::new(key, entry).into())
        } else {
            None
        }
    }

    /// Return a resource in a default state with a specific Id
    pub fn new_resource_untyped(&self, id: ResourceTypeAndId) -> Option<HandleUntyped> {
        if let Some(processor) = self.processors.get(&id.kind) {
            let resource = processor.new_resource();

            let entry = HandleEntry::new(id, resource, self.weak_self());
            let mut guard = self.write_inner();
            let key = guard.handles.insert(entry.clone());
            guard.index.insert(id.id, key);
            Some(HandleUntyped::new(key, entry))
        } else {
            None
        }
    }

    /// Return the available resource type that can be created
    pub fn get_resource_types(&self) -> Vec<(ResourceType, &'static str)> {
        self.processors
            .iter()
            .map(|(k, _processor)| (*k, k.as_pretty()))
            .collect()
    }

    /// Interface to initialize a new `Resource` from a stream
    /// # Errors
    /// Will return `AssetRegistryError` if the resource was not deserialized properly
    pub async fn deserialize_resource(
        &self,
        id: ResourceTypeAndId,
        reader: AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut request = LoadRequest::new(id, self.arc_self());
        return self.loader.load_from_stream(id, reader, &mut request).await;
    }

    /// Interface to serialize a `Resource` into a stream
    /// # Errors
    /// Will return `AssetRegistryError` if the resource was not serialize properly
    pub fn serialize_resource(
        &self,
        handle: impl AsRef<HandleUntyped>,
        writer: &mut dyn std::io::Write,
    ) -> Result<(usize, Vec<ResourcePathId>), AssetRegistryError> {
        let guard = self.read_inner();
        let entry = guard
            .handles
            .get(handle.as_ref().key())
            .ok_or_else(|| AssetRegistryError::InvalidHandle(handle.as_ref().key()))?;

        if let Some(processor) = self.processors.get(&entry.id.kind) {
            let resource = entry.asset.read().unwrap();
            let resource = &resource;

            let build_deps = processor.extract_build_dependencies(resource.as_ref());
            let written = processor.write_resource(resource.as_ref(), writer)?;
            Ok((written, build_deps))
        } else {
            Err(AssetRegistryError::ProcessorNotFound(entry.id.kind))
        }
    }

    // Delayed manifest load, will look up in all devices
    //pub fn load_manifest(&self, _manifest_id: &Identifier) {
    //    unimplemented!();
    //}
}
