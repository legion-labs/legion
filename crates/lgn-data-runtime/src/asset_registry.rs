use std::{
    collections::HashMap,
    path::Path,
    pin::Pin,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

use futures::{future::Shared, Future, FutureExt};

use lgn_content_store::{
    indexing::{SharedTreeIdentifier, TreeIdentifier},
    Provider,
};
use lgn_ecs::schedule::SystemLabel;
use lgn_tracing::{debug, error};
use slotmap::SlotMap;

use crate::{
    asset_loader::AssetLoaderIO, vfs, AssetRegistryHandleKey, ComponentInstaller, EditHandle,
    EditHandleUntyped, Handle, HandleEntry, HandleUntyped, Resource, ResourceId, ResourceInstaller,
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
    #[error("Asset registry failed to serialize: '{0}'")]
    SerializationFailed(&'static str, String),

    /// General IO Error when loading a resource
    #[error("IO error: {0}")]
    IOError(std::sync::Arc<std::io::Error>),

    /// AssetLoader for a type not present
    #[error("ResourceInstaller for ResourceType '{0:?}' not found")]
    ResourceInstallerNotFound(ResourceType),

    /// Processor not found
    #[error("ResourceType '{0}'not found")]
    ResourceTypeNotRegistered(ResourceType),

    /// AssetLoaderError fallthrough
    #[error("Asset registry reflection Error '{0}'")]
    ReflectionError(#[from] lgn_data_model::ReflectionError),

    /// AssetLoaderError fallthrough
    #[error("{0}")]
    Generic(String),
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
    resource_installers: HashMap<ResourceType, Arc<dyn ResourceInstaller>>,
    component_installers: HashMap<std::any::TypeId, Arc<dyn ComponentInstaller>>,

    devices: Vec<Box<dyn vfs::Device + Send>>,
}

impl AssetRegistryOptions {
    /// Creates a blank set of options for [`AssetRegistry`] configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            resource_installers: HashMap::new(),
            component_installers: HashMap::new(),
            devices: vec![],
        }
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

    /// Adds a device that can read resources.
    #[must_use]
    pub fn add_device(mut self, device: Box<dyn vfs::Device + Send>) -> Self {
        self.devices.push(device);
        self
    }

    /// Specifying `content-addressable storage device` will mount a device that
    /// allows to read resources from a specified content store through
    /// provided manifest.
    #[must_use]
    pub fn add_device_cas(
        self,
        volatile_provider: Arc<Provider>,
        runtime_manifest_id: SharedTreeIdentifier,
    ) -> Self {
        self.add_device(Box::new(vfs::CasDevice::new(
            volatile_provider,
            runtime_manifest_id,
        )))
    }

    /// Specifying `build device` will mount a device that allows to build
    /// resources as they are being requested.
    ///
    /// `force_recompile` if set will cause each load request to go through data
    /// compilation.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_device_build(
        self,
        volatile_provider: Arc<Provider>,
        source_manifest_id: SharedTreeIdentifier,
        runtime_manifest_id: Option<TreeIdentifier>,
        build_bin: impl AsRef<Path>,
        output_db_addr: &str,
        repository_name: &str,
        branch_name: &str,
        force_recompile: bool,
    ) -> Self {
        self.add_device(Box::new(
            vfs::BuildDevice::new(
                volatile_provider,
                source_manifest_id,
                runtime_manifest_id,
                build_bin,
                output_db_addr,
                repository_name,
                branch_name,
                force_recompile,
            )
            .await,
        ))
    }

    /// Creates [`AssetRegistry`] based on `AssetRegistryOptions`.
    pub async fn create(self) -> Arc<AssetRegistry> {
        Arc::new(AssetRegistry {
            inner: RwLock::new(Inner {
                handles: SlotMap::with_key(),
                index: HashMap::new(),
            }),
            loader: AssetLoaderIO::new(self.devices, self.resource_installers),
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

/// Message to notifying `AssetRegistry` Operation
pub enum AssetRegistryMessage {
    /// Sent when resources changed
    ChangedResources(Vec<ResourceTypeAndId>),
}

/// Async reader type for `AssetRegistry`/`AssetLoader`
pub type AssetRegistryReader = Pin<Box<dyn tokio::io::AsyncRead + Send>>;

impl AssetRegistry {
    fn write_inner(&self) -> RwLockWriteGuard<'_, Inner> {
        self.inner.write().unwrap()
    }

    /// Trigger a reload of a given primary resource.
    /// # Errors
    /// Return `AssetRegistryError` on failure
    pub async fn reload(
        &self,
        resource_id: ResourceTypeAndId,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let future = self.new_load_request(resource_id);
        future.await
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

    // TODO: replace with Arc<Inner>
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

    /// Interface to serialize a `Resource` into a stream
    /// # Errors
    /// Will return `AssetRegistryError` if the resource was not serialize properly
    /*pub fn serialize_resource_without_dependencies(
        &self,
        kind: ResourceType,
        handle: impl AsRef<HandleUntyped>,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, AssetRegistryError> {
        let mut guard = self.write_inner();
        let inner: &mut Inner = &mut guard;

        if let Some(processor) = self.processors.write().unwrap().get_mut(&kind) {
            let resource = inner
                .assets
                .get(&handle.as_ref().id())
                .ok_or_else(|| AssetRegistryError::ResourceNotFound(handle.as_ref().id()))?
                .as_ref();

            let written = processor.write_resource(&*resource, writer)?;
            Ok(written)
        } else {
            Err(AssetRegistryError::ProcessorNotFound(kind))
        }
    }

    /// Interface to serialize a `Resource` into a stream
    /// # Errors
    /// Will return `AssetRegistryError` if the resource was not serialize properly
    pub fn get_build_dependencies(
        &self,
        kind: ResourceType,
        handle: impl AsRef<HandleUntyped>,
    ) -> Result<Vec<ResourcePathId>, AssetRegistryError> {
        let mut guard = self.write_inner();
        let inner: &mut Inner = &mut guard;

        if let Some(processor) = self.processors.write().unwrap().get_mut(&kind) {
            let resource = inner
                .assets
                .get(&handle.as_ref().id())
                .ok_or_else(|| AssetRegistryError::ResourceNotFound(handle.as_ref().id()))?
                .as_ref();

            let build_deps = processor.extract_build_dependencies(&*resource);
            Ok(build_deps)
        } else {
            Err(AssetRegistryError::ProcessorNotFound(kind))
        }
    }*/

    /// Create a Clone that can be edited and committed.
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
    ) -> Option<&Arc<dyn ComponentInstaller>> {
        self.component_installers.get(&type_id)
    }

    /// Unloads assets based on their reference counts.
    pub fn update(&self) {
        self.collect_dropped_handles();
    }

    // Delayed manifest load, will look up in all devices
    //pub fn load_manifest(&self, _manifest_id: &Identifier) {
    //    unimplemented!();
    //}
}

/*
#[cfg(test)]
mod tests {
    use std::panic;

    use generic_data::offline::TestAsset;
    use lgn_content_store::indexing::{ResourceIndex, ResourceWriter};

    use super::*;
    use crate::{new_resource_type_and_id_indexer, ResourceId};

    async fn setup_singular_asset_test(content: &[u8]) -> (ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest = ResourceIndex::new_exclusive(
            Arc::clone(&data_provider),
            new_resource_type_and_id_indexer(),
        )
        .await;

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let provider_id = data_provider
                .write_resource_from_bytes(content)
                .await
                .unwrap();
            manifest
                .add_resource(&type_id.into(), provider_id)
                .await
                .unwrap();

            type_id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(data_provider, SharedTreeIdentifier::new(manifest.id()))
            .add_loader::<test_asset::TestAsset>()
            .create()
            .await;

        (asset_id, reg)
    }

    async fn setup_dependency_test() -> (ResourceTypeAndId, ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest = ResourceIndex::new_exclusive(
            Arc::clone(&data_provider),
            new_resource_type_and_id_indexer(),
        )
        .await;

        const BINARY_PARENT_ASSETFILE: [u8; 100] = [
            97, 115, 102, 116, // header (asft)
            1, 0, // version
            1, 0, 0, 0, 0, 0, 0, 0, // references count
            0x9c, 0x44, 0xd9, 0x53, 0x0e, 0x17, 0x63, 0xf0, // first reference (ResourceType)
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0xf0, 0, 0, 0, 0, 0, 0, // fist reference (ResourceId)
            0x9c, 0x44, 0xd9, 0x53, 0x0e, 0x17, 0x63,
            0xf0, // first asset type (RessourceType)
            1, 0, 0, 0, 0, 0, 0, 0, // assets count following in stream
            38, 0, 0, 0, 0, 0, 0, 0, // bytes for next asset data
            6, 0, 0, 0, 0, 0, 0, 0, 112, 97, 114, 101, 110, 116, 0x9c, 0x44, 0xd9, 0x53, 0x0e,
            0x17, 0x63, 0xf0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0xf0, 0, 0, 0, 0, 0, 0, // asset data
        ];
        const BINARY_CHILD_ASSETFILE: [u8; 67] = [
            97, 115, 102, 116, // header (asft)
            1, 0, // version
            0, 0, 0, 0, 0, 0, 0, 0, // references count (none here)
            0x9c, 0x44, 0xd9, 0x53, 0x0e, 0x17, 0x63,
            0xf0, // first asset type (RessourceType)
            1, 0, 0, 0, 0, 0, 0, 0, // assets count following in stream
            29, 0, 0, 0, 0, 0, 0, 0, // bytes for next asset data
            5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // asset data
        ];

        let child_id = ResourceTypeAndId {
            kind: refs_asset::RefsAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };

        let parent_id = {
            manifest
                .add_resource(
                    &child_id.into(),
                    data_provider
                        .write_resource_from_bytes(&BINARY_CHILD_ASSETFILE)
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            let provider_id = data_provider
                .write_resource_from_bytes(&BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            let type_id = ResourceTypeAndId {
                kind: refs_asset::RefsAsset::TYPE,
                id: ResourceId::new_explicit(2),
            };
            manifest
                .add_resource(&type_id.into(), provider_id)
                .await
                .unwrap();
            type_id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(data_provider, SharedTreeIdentifier::new(manifest.id()))
            .add_loader::<refs_asset::RefsAsset>()
            .create()
            .await;

        (parent_id, child_id, reg)
    }

    const BINARY_RAWFILE: [u8; 5] = [99, 104, 105, 108, 100];

    #[tokio::test]
    async fn load_assetfile() {
        let (asset_id, reg) =
            setup_singular_asset_test(&crate::test_asset::tests::BINARY_ASSETFILE).await;

        let internal_id;
        {
            let a = reg.load_untyped(asset_id);
            internal_id = a.id();

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_loaded(&reg) {
                let sleep_time = Duration::from_millis(10);
                tokio::time::sleep(sleep_time).await;
                test_timeout -= sleep_time;
                reg.update();
            }

            assert!(a.is_loaded(&reg));
            assert!(!a.is_err(&reg));
            assert!(reg.is_loaded(internal_id));
            {
                let b = a.clone();
                reg.update();
                assert_eq!(a, b);

                assert!(b.is_loaded(&reg));
                assert!(!b.is_err(&reg));
                assert!(reg.is_loaded(internal_id));
            }
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }

    #[tokio::test]
    async fn load_rawfile() {
        let (asset_id, reg) = setup_singular_asset_test(&BINARY_RAWFILE).await;

        let internal_id;
        {
            let a = reg.load_untyped(asset_id);
            internal_id = a.id();

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_loaded(&reg) {
                let sleep_time = Duration::from_millis(10);
                tokio::time::sleep(sleep_time).await;
                test_timeout -= sleep_time;
                reg.update();
            }

            assert!(a.is_loaded(&reg));
            assert!(!a.is_err(&reg));
            assert!(reg.is_loaded(internal_id));
            {
                let b = a.clone();
                reg.update();
                assert_eq!(a, b);

                assert!(b.is_loaded(&reg));
                assert!(!b.is_err(&reg));
                assert!(reg.is_loaded(internal_id));
            }
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }

    #[tokio::test]
    async fn load_error() {
        let (_, reg) = setup_singular_asset_test(&crate::test_asset::tests::BINARY_ASSETFILE).await;

        let internal_id;
        {
            let a = reg.load_untyped(ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(7),
            });
            internal_id = a.id();

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_err(&reg) {
                let sleep_time = Duration::from_millis(10);
                tokio::time::sleep(sleep_time).await;
                test_timeout -= sleep_time;
                reg.update();
            }

            assert!(!a.is_loaded(&reg));
            assert!(a.is_err(&reg));
            assert!(!reg.is_loaded(internal_id));
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }

    #[tokio::test]
    async fn load_error_sync() {
        let (_, reg) = setup_singular_asset_test(&crate::test_asset::tests::BINARY_ASSETFILE).await;

        let internal_id;
        {
            let a = reg
                .load_untyped_async(ResourceTypeAndId {
                    kind: test_asset::TestAsset::TYPE,
                    id: ResourceId::new_explicit(7),
                })
                .await;
            internal_id = a.id();

            assert!(!a.is_loaded(&reg));
            assert!(a.is_err(&reg));
            assert!(!reg.is_loaded(internal_id));
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }

    #[tokio::test]
    async fn load_dependency() {
        let (parent_id, child_id, reg) = setup_dependency_test().await;

        let parent = reg.load_untyped_async(parent_id).await;
        assert!(parent.is_loaded(&reg));

        let child = reg.get_untyped(child_id).expect("be loaded indirectly");
        assert!(
            child.is_loaded(&reg),
            "The dependency should immediately be considered as loaded"
        );

        std::mem::drop(parent);
        reg.update();

        assert!(reg.get_untyped(parent_id).is_none());

        assert!(
            child.is_loaded(&reg),
            "The dependency should be kept alive because of the handle"
        );

        std::mem::drop(child);
        reg.update();
        assert!(reg.get_untyped(child_id).is_none());
    }

    #[tokio::test]
    async fn loaded_notification() {
        let (asset_id, reg) =
            setup_singular_asset_test(&crate::test_asset::tests::BINARY_ASSETFILE).await;

        let mut notif = reg.subscribe_to_load_events();
        {
            let _handle = reg.load_untyped_async(asset_id).await;
            reg.update();
        } // user handle drops here..

        reg.update();
        assert!(reg.is_loaded(asset_id)); // ..but ResourceLoadEvent::Loaded still holds the reference.

        match notif.try_recv() {
            Ok(ResourceLoadEvent::Loaded(loaded)) => {
                assert_eq!(loaded.id(), asset_id);
                assert!(loaded.is_loaded(&reg));
            }
            _ => panic!(),
        }
        reg.update();
        assert!(!reg.is_loaded(asset_id));
    }

    #[tokio::test]
    async fn reload_no_change() {
        let (asset_id, reg) =
            setup_singular_asset_test(&crate::test_asset::tests::BINARY_ASSETFILE).await;

        let internal_id;
        {
            let a = reg.load_untyped_async(asset_id).await;
            internal_id = a.id();

            assert!(a.is_loaded(&reg));
            assert!(!a.is_err(&reg));

            let mut notif = reg.subscribe_to_load_events();
            assert!(reg.reload(a.id()));

            let mut test_timeout = Duration::from_millis(500);
            let dt = Duration::from_millis(10);

            while test_timeout > Duration::ZERO {
                reg.update();
                if let Ok(Some(ResourceLoadEvent::Reloaded(reloaded))) =
                    tokio::time::timeout(dt, notif.recv()).await
                {
                    assert_eq!(a, reloaded);
                    break;
                }
                test_timeout = test_timeout.saturating_sub(dt);
            }
            assert!(test_timeout > Duration::ZERO);
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }
}
*/
