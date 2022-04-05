use lgn_content_store2::ContentProvider;
use lgn_ecs::schedule::SystemLabel;
use std::{
    any::Any,
    cell::Cell,
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

use crate::{
    asset_loader::{create_loader, AssetLoaderStub, LoaderResult},
    manifest::Manifest,
    vfs, Asset, AssetLoader, AssetLoaderError, Handle, HandleUntyped, Resource, ResourceType,
    ResourceTypeAndId,
};

/// Error type for Asset Registry
#[derive(thiserror::Error, Debug)]
pub enum AssetRegistryError {
    /// Error when a resource failed to load
    #[error(
        "Dependent Resource '{resource:?}' failed loading because '{parent:?}': {parent_error}"
    )]
    ResourceDependentLoadFailed {
        /// Resource try to load
        resource: ResourceTypeAndId,
        /// Parent resource that failed
        parent: ResourceTypeAndId,
        /// Inner error of the parent
        parent_error: String,
    },

    /// Error when a resource is not found
    #[error("Resource '{0:?}' was not found")]
    ResourceNotFound(ResourceTypeAndId),

    /// General IO Error when loading a resource
    #[error("Resource '{0:?}' IO error: {1}")]
    ResourceIOError(ResourceTypeAndId, std::io::Error),

    /// Type mismatched
    #[error("Resource '{0:?}' type mistmached: {1} expected {2}")]
    ResourceTypeMismatch(ResourceTypeAndId, String, String),

    /// Version mismatched
    #[error("Resource '{0:?}' type mistmached: {1} expected {2}")]
    ResourceVersionMismatch(ResourceTypeAndId, u16, u16),

    /// AssetLoader for a type not present
    #[error("AssetLoader for ResourceType '{0}' not found")]
    AssetLoaderNotFound(ResourceType),

    /// AssetLoader for a type not present
    #[error("Resource '{0:?}' failed to load. {1}")]
    AssetLoaderFailed(ResourceTypeAndId, AssetLoaderError),

    /// General IO Error
    #[error("IO Error {0}")]
    IOError(String),

    /// General IO Error
    #[error("Invalid Data: {0}")]
    InvalidData(String),
}

/// Return a Guarded Ref to a Asset
pub struct AssetRegistryGuard<'a, T: ?Sized + 'a> {
    //lock: &'a RwLock<Inner>,
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
    loaders: HashMap<ResourceType, Box<dyn AssetLoader + Send + Sync>>,
    devices: Vec<Box<(dyn vfs::Device + Send)>>,
}

impl AssetRegistryOptions {
    /// Creates a blank set of options for [`AssetRegistry`] configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
            devices: vec![],
        }
    }

    /// Specifying `directory device` will mount a device that allows to read
    /// resources from a specified directory.
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
    pub fn add_device_cas(
        mut self,
        content_store: Arc<Box<dyn ContentProvider + Send + Sync>>,
        manifest: Manifest,
    ) -> Self {
        self.devices
            .push(Box::new(vfs::CasDevice::new(manifest, content_store)));
        self
    }

    /// Specifying `build device` will mount a device that allows to build
    /// resources as they are being requested.
    ///
    /// `force_recompile` if set will cause each load request to go through data
    /// compilation.
    #[allow(clippy::too_many_arguments)]
    pub fn add_device_build(
        mut self,
        content_store: Arc<Box<dyn ContentProvider + Send + Sync>>,
        manifest: Manifest,
        build_bin: impl AsRef<Path>,
        output_db_addr: String,
        project: impl AsRef<Path>,
        force_recompile: bool,
    ) -> Self {
        self.devices.push(Box::new(vfs::BuildDevice::new(
            manifest,
            content_store,
            build_bin,
            output_db_addr,
            project,
            force_recompile,
        )));
        self
    }

    /// Enables support of a given [`Resource`] by adding corresponding
    /// [`AssetLoader`].
    pub fn add_loader<A: Asset>(mut self) -> Self {
        ResourceType::register_name(A::TYPE, A::TYPENAME);
        self.loaders.insert(A::TYPE, Box::new(A::Loader::default()));
        self
    }

    /// Enables support of a given [`Resource`] by adding corresponding
    /// [`AssetLoader`].
    pub fn add_loader_mut<A: Asset>(&mut self) -> &mut Self {
        ResourceType::register_name(A::TYPE, A::TYPENAME);
        self.loaders.insert(A::TYPE, Box::new(A::Loader::default()));
        self
    }

    /// Creates [`AssetRegistry`] based on `AssetRegistryOptions`.
    pub async fn create(self) -> Arc<AssetRegistry> {
        let (loader, mut io) = create_loader(self.devices);

        let registry = Arc::new(AssetRegistry {
            inner: RwLock::new(Inner {
                assets: HashMap::new(),
                load_errors: HashMap::new(),
                load_event_senders: Vec::new(),
                loader,
            }),
            load_thread: Cell::new(None),
        });

        for (kind, mut loader) in self.loaders {
            loader.register_registry(registry.clone());
            io.register_loader(kind, loader);
        }

        let rt = tokio::runtime::Handle::current();

        let load_thread = rt.spawn(async move {
            let mut loader = io;
            while loader.wait(Duration::from_millis(100)).await.is_some() {}
        });

        registry.load_thread.set(Some(load_thread));

        registry
    }
}

struct Inner {
    assets: HashMap<ResourceTypeAndId, Box<dyn Any + Send + Sync>>,
    loader: AssetLoaderStub,
    load_errors: HashMap<ResourceTypeAndId, AssetRegistryError>,
    load_event_senders: Vec<tokio::sync::mpsc::UnboundedSender<ResourceLoadEvent>>,
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
    load_thread: Cell<Option<tokio::task::JoinHandle<()>>>,
}

/// Label to use for scheduling systems that require the `AssetRegistry`
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum AssetRegistryScheduling {
    /// AssetRegistry has been created
    AssetRegistryCreated,
}

/// Event for `AssetRegistry` operation
#[allow(clippy::enum_variant_names)]
pub enum AssetRegistryEvent {
    /// Notify that a resource has been loaded
    AssetLoaded(ResourceTypeAndId),

    /// Notify that a resource has been unloaded
    AssetUnloaded(ResourceTypeAndId),

    /// Notify that a resource has changed
    AssetChanged(ResourceTypeAndId),
}

/// A resource loading event is emitted when a resource is loaded, unloaded, or
/// loading fails
#[derive(Debug, Clone)]
pub enum ResourceLoadEvent {
    /// Successful resource load, resulting from either a handle load, or the
    /// loading of a dependency
    Loaded(HandleUntyped),
    /// Resource unload event
    Unloaded(ResourceTypeAndId),
    /// Sent when a loading attempt has failed
    LoadError(ResourceTypeAndId, String),
    /// Successful resource reload
    Reloaded(HandleUntyped),
}

impl Drop for AssetRegistry {
    fn drop(&mut self) {
        self.write_inner().loader.terminate();
        self.load_thread.take().unwrap().abort();
    }
}

/// Safety: it is safe share references to `AssetRegistry` between threads
/// and the implementation will panic! if its safety rules are not fulfilled.
unsafe impl Sync for AssetRegistry {}

impl AssetRegistry {
    fn read_inner(&self) -> RwLockReadGuard<'_, Inner> {
        self.inner.read().unwrap()
    }

    fn write_inner(&self) -> RwLockWriteGuard<'_, Inner> {
        self.inner.write().unwrap()
    }

    /// Requests an asset load.
    ///
    /// The asset will be unloaded after all instances of [`HandleUntyped`] and
    /// [`Handle`] that refer to that asset go out of scope.
    ///
    /// This is a non-blocking call.
    /// For a blocking version see [`Self::load_untyped_sync`] and [`Self::load_untyped_async`].
    pub fn load_untyped(&self, type_id: ResourceTypeAndId) -> HandleUntyped {
        self.write_inner().loader.load(type_id)
    }

    /// Trigger a reload of a given primary resource.
    pub fn reload(&self, type_id: ResourceTypeAndId) -> bool {
        self.write_inner().loader.reload(type_id)
    }

    /// Returns a handle to the resource if a handle to this resource already
    /// exists.
    pub fn get_untyped(&self, type_id: ResourceTypeAndId) -> Option<HandleUntyped> {
        self.write_inner().loader.get_handle(type_id)
    }

    /// Same as [`Self::load_untyped`] but blocks until the resource load
    /// completes or a load error occurs.
    pub fn load_untyped_sync(&self, type_id: ResourceTypeAndId) -> HandleUntyped {
        let handle = self.load_untyped(type_id);
        // todo: instead of polling this could use 'condvar' or similar.
        while !handle.is_loaded(self) && !handle.is_err(self) {
            self.update();
            std::thread::sleep(Duration::from_micros(100));
        }

        handle
    }

    /// Same as [`Self::load_untyped`] but waits until the resource load
    /// completes or a load error occurs.
    pub async fn load_untyped_async(&self, type_id: ResourceTypeAndId) -> HandleUntyped {
        let handle = self.load_untyped(type_id);
        while !handle.is_loaded(self) && !handle.is_err(self) {
            self.update();
            // todo: instead of sleeping a better solution would be to use something like 'waitmap'.
            tokio::time::sleep(Duration::from_micros(100)).await;
        }

        handle
    }

    /// Same as [`Self::load_untyped`] but the returned handle is generic over
    /// asset type `T` for convenience.
    pub fn load<T: Any + Resource>(&self, id: ResourceTypeAndId) -> Handle<T> {
        let handle = self.load_untyped(id);
        Handle::<T>::from(handle)
    }

    /// Same as [`Self::load`] but blocks until the resource load completes or
    /// returns an error.
    pub fn load_sync<T: Any + Resource>(&self, id: ResourceTypeAndId) -> Handle<T> {
        let handle = self.load_untyped_sync(id);
        Handle::<T>::from(handle)
    }

    /// Same as [`Self::load`] but waits until the resource load completes or
    /// returns an error.
    pub async fn load_async<T: Any + Resource>(&self, id: ResourceTypeAndId) -> Handle<T> {
        let handle = self.load_untyped_async(id).await;
        Handle::<T>::from(handle)
    }

    /// Retrieves a reference to an asset, None if asset is not loaded.
    pub(crate) fn get<T: Any + Resource>(
        &self,
        id: ResourceTypeAndId,
    ) -> Option<AssetRegistryGuard<'_, T>> {
        let guard = self.inner.read().unwrap();
        let inner: &Inner = &guard;
        if let Some(asset) = inner.assets.get(&id) {
            if let Some(ptr) = asset.downcast_ref::<T>().map(|c| c as *const T) {
                return Some(AssetRegistryGuard { _guard: guard, ptr });
            }
        }
        None
    }

    /// Tests if an asset is loaded.
    pub fn is_loaded(&self, id: ResourceTypeAndId) -> bool {
        self.read_inner().assets.get(&id).is_some()
    }

    /// Unloads assets based on their reference counts.
    pub fn update(&self) {
        let mut load_events = Vec::new();

        {
            let mut inner = self.write_inner();
            for removed_id in inner.loader.collect_dropped_handles() {
                inner.load_errors.remove(&removed_id);
                inner.assets.remove(&removed_id);
                inner.loader.unload(removed_id);
            }

            while let Some(result) = inner.loader.try_result() {
                // todo: add success/failure callbacks using the provided LoadId.
                match result {
                    LoaderResult::Loaded(handle, resource, _load_id) => {
                        inner.assets.insert(handle.id(), resource);
                        load_events.push(ResourceLoadEvent::Loaded(handle));
                    }
                    LoaderResult::Unloaded(id) => {
                        inner.assets.remove(&id);
                        load_events.push(ResourceLoadEvent::Unloaded(id));
                    }
                    LoaderResult::LoadError(handle, _load_id, error_kind) => {
                        load_events.push(ResourceLoadEvent::LoadError(
                            handle.id(),
                            error_kind.to_string(),
                        ));
                        inner.load_errors.insert(handle.id(), error_kind);
                    }
                    LoaderResult::Reloaded(handle, resource) => {
                        let old_resource = inner.assets.insert(handle.id(), resource);
                        assert!(old_resource.is_some());
                        load_events.push(ResourceLoadEvent::Reloaded(handle));
                    }
                }
            }
        }

        {
            // broadcast load events
            let inner = self.read_inner();
            for sender in &inner.load_event_senders {
                for event in &load_events {
                    sender.send(event.clone()).unwrap();
                }
            }
        }
    }

    pub(crate) fn is_err(&self, type_id: ResourceTypeAndId) -> bool {
        self.read_inner().load_errors.contains_key(&type_id)
    }

    /// Returns the last load error for a resource type
    pub fn retrieve_err(&self, type_id: ResourceTypeAndId) -> Option<AssetRegistryError> {
        self.write_inner().load_errors.remove(&type_id)
    }

    /// Subscribe to load events, to know when resources are loaded and
    /// unloaded. Returns a channel receiver that will receive
    /// `ResourceLoadEvent`s.
    pub fn subscribe_to_load_events(
        &self,
    ) -> tokio::sync::mpsc::UnboundedReceiver<ResourceLoadEvent> {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ResourceLoadEvent>();
        self.write_inner().load_event_senders.push(sender);
        receiver
    }
}

#[cfg(test)]
mod tests {
    use lgn_content_store2::{ContentWriterExt, ProviderConfig};

    use crate::ResourceId;

    mod refs_asset {
        //! This module defines a test asset.
        //!
        //! It is used to test the data compilation process until we have a
        //! proper asset available.

        use std::{any::Any, io, sync::Arc};

        use byteorder::{LittleEndian, ReadBytesExt};

        use crate::{
            resource, Asset, AssetLoader, AssetLoaderError, AssetRegistry, Reference, Resource,
            ResourceId, ResourceType, ResourceTypeAndId,
        };
        /// Asset temporarily used for testing.
        ///
        /// To be removed once real asset types exist.
        #[resource("refs_asset")]
        pub struct RefsAsset {
            /// Test content.
            pub content: String,
            pub reference: Option<Reference<RefsAsset>>,
        }

        impl Asset for RefsAsset {
            type Loader = RefsAssetLoader;
        }

        /// [`RefsAsset`]'s asset creator temporarily used for testings.
        ///
        /// To be removed once real asset types exists.
        #[derive(Default)]
        pub struct RefsAssetLoader {
            registry: Option<Arc<AssetRegistry>>,
        }

        impl AssetLoader for RefsAssetLoader {
            fn load(
                &mut self,
                reader: &mut dyn io::Read,
            ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
                let len = reader.read_u64::<LittleEndian>()?;

                let mut content = vec![0; len as usize];
                reader.read_exact(&mut content)?;

                let reference = read_maybe_reference::<RefsAsset>(reader)?;
                let asset = Box::new(RefsAsset {
                    content: String::from_utf8(content).unwrap(),
                    reference,
                });
                Ok(asset)
            }

            fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
                let asset = asset.downcast_mut::<RefsAsset>().unwrap();
                if let Some(reference) = &mut asset.reference {
                    reference.activate(self.registry.as_ref().unwrap());
                }
            }
            fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
                self.registry = Some(registry);
            }
        }

        fn read_maybe_reference<T>(
            reader: &mut dyn std::io::Read,
        ) -> Result<Option<Reference<T>>, std::io::Error>
        where
            T: Any + Resource,
        {
            let underlying_type = reader.read_u64::<LittleEndian>()?;
            if underlying_type == 0 {
                return Ok(None);
            }
            let underlying_id = reader.read_u128::<LittleEndian>()?;
            if underlying_id == 0 {
                return Ok(None);
            }
            Ok(Some(Reference::Passive(ResourceTypeAndId {
                kind: ResourceType::from_raw(underlying_type),
                id: ResourceId::from_raw(underlying_id),
            })))
        }
    }

    use std::panic;

    use super::*;
    use crate::test_asset;

    async fn setup_singular_asset_test(content: &[u8]) -> (ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_content_store = Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_store.write_content(content).await.unwrap();
            manifest.insert(type_id, checksum);
            type_id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(data_content_store, manifest)
            .add_loader::<test_asset::TestAsset>()
            .create()
            .await;

        (asset_id, reg)
    }

    async fn setup_dependency_test() -> (ResourceTypeAndId, ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_content_store = Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

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
            manifest.insert(
                child_id,
                data_content_store
                    .write_content(&BINARY_CHILD_ASSETFILE)
                    .await
                    .unwrap(),
            );
            let checksum = data_content_store
                .write_content(&BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            let id = ResourceTypeAndId {
                kind: refs_asset::RefsAsset::TYPE,
                id: ResourceId::new_explicit(2),
            };
            manifest.insert(id, checksum);
            id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(data_content_store, manifest)
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
