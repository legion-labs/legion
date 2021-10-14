use std::{
    any::Any,
    collections::HashMap,
    io,
    path::Path,
    thread::{self, JoinHandle},
    time::Duration,
};

use legion_content_store::ContentStore;

use crate::{
    asset_loader::{create_loader, AssetLoaderStub, LoaderResult},
    manifest::Manifest,
    vfs, Asset, AssetLoader, Handle, HandleId, HandleUntyped, Resource, ResourceId, ResourceType,
};

/// Options which can be used to configure the creation of [`AssetRegistry`].
pub struct AssetRegistryOptions {
    loaders: HashMap<ResourceType, Box<dyn AssetLoader + Send>>,
    devices: Vec<Box<dyn vfs::Device>>,
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

    /// Specifying `directory device` will mount a device that allows to read resources
    /// from a specified directory.
    pub fn add_device_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.devices.push(Box::new(vfs::DirDevice::new(path)));
        self
    }

    /// Specifying `content-addressable storage device` will mount a device that allows
    /// to read resources from a specified content store through provided manifest.
    pub fn add_device_cas(
        mut self,
        content_store: Box<dyn ContentStore>,
        manifest: Manifest,
    ) -> Self {
        self.devices
            .push(Box::new(vfs::CasDevice::new(manifest, content_store)));
        self
    }

    /// Enables support of a given [`Resource`] by adding corresponding [`AssetLoader`].
    pub fn add_loader<A: Asset>(mut self) -> Self {
        self.loaders.insert(A::TYPE, Box::new(A::Loader::default()));
        self
    }

    /// Creates [`AssetRegistry`] based on `AssetRegistryOptions`.
    pub fn create(self) -> AssetRegistry {
        let (loader, mut io) = create_loader(self.devices);

        for (kind, loader) in self.loaders {
            io.register_loader(kind, loader);
        }

        let load_thread = thread::spawn(move || {
            let mut loader = io;
            while loader.wait(Duration::from_millis(100)).is_some() {}
        });

        AssetRegistry {
            assets: HashMap::new(),
            load_errors: HashMap::new(),
            load_thread: Some(load_thread),
            loader,
        }
    }
}

/// Registry of all loaded [`Resource`]s.
///
/// Provides an API to load assets by their [`ResourceId`]. The lifetime of an [`Resource`] is determined
/// by the reference counted [`HandleUntyped`] and [`Handle`].
///
/// [`Handle`]: [`crate::Handle`]
pub struct AssetRegistry {
    assets: HashMap<ResourceId, Box<dyn Any + Send + Sync>>,
    load_errors: HashMap<ResourceId, io::ErrorKind>,
    load_thread: Option<JoinHandle<()>>,
    loader: AssetLoaderStub,
}

impl Drop for AssetRegistry {
    fn drop(&mut self) {
        self.loader.terminate();
        self.load_thread.take().unwrap().join().unwrap();
    }
}

impl AssetRegistry {
    /// Requests an asset load.
    ///
    /// The asset will be unloaded after all instances of [`HandleUntyped`] and
    /// [`Handle`] that refer to that asset go out of scope.
    pub fn load_untyped(&mut self, id: ResourceId) -> HandleUntyped {
        self.loader.load(id)
    }

    /// Trigger a reload of a given primary resource.
    pub fn reload(&mut self, id: ResourceId) -> bool {
        self.loader.reload(id)
    }

    /// Returns a handle to the resource if a handle to this resource already exists.
    pub fn get_untyped(&mut self, id: ResourceId) -> Option<HandleUntyped> {
        self.loader.get_handle(id)
    }

    /// Same as [`Self::load_untyped`] but blocks until the resource load completes or returns an error.
    pub fn load_untyped_sync(&mut self, id: ResourceId) -> HandleUntyped {
        let handle = self.loader.load(id);
        // todo: this will be improved with async/await
        while !handle.is_loaded(self) && !handle.is_err(self) {
            self.update();
            std::thread::sleep(Duration::from_micros(100));
        }

        handle
    }

    /// Same as [`Self::load_untyped`] but the returned handle is generic over asset type `T` for convenience.
    pub fn load<T: Any + Resource>(&mut self, id: ResourceId) -> Handle<T> {
        let handle = self.load_untyped(id);
        Handle::<T>::from(handle)
    }

    /// Same as [`Self::load`] but blocks until the resource load completes or returns an error.
    pub fn load_sync<T: Any + Resource>(&mut self, id: ResourceId) -> Handle<T> {
        let handle = self.load_untyped_sync(id);
        Handle::<T>::from(handle)
    }

    /// Retrieves the asset id associated with a handle.
    pub(crate) fn get_asset_id(&self, handle_id: HandleId) -> Option<ResourceId> {
        self.loader.get_asset_id(handle_id)
    }

    /// Retrieves a reference to an asset, None if asset is not loaded.
    pub(crate) fn get<T: Any + Resource>(&self, handle_id: HandleId) -> Option<&T> {
        if let Some(asset_id) = self.get_asset_id(handle_id) {
            if let Some(asset) = self.assets.get(&asset_id) {
                return asset.downcast_ref::<T>();
            }
        }
        None
    }

    /// Tests if an asset is loaded.
    pub(crate) fn is_loaded(&self, handle_id: HandleId) -> bool {
        if let Some(asset_id) = self.get_asset_id(handle_id) {
            return self.assets.get(&asset_id).is_some();
        }
        false
    }

    /// Unloads assets based on their reference counts.
    pub fn update(&mut self) {
        while let Some(removed_id) = self.loader.process_refcount_ops() {
            self.load_errors.remove(&removed_id);
            self.assets.remove(&removed_id);
            self.loader.unload(removed_id);
        }

        while let Some(result) = self.loader.try_result() {
            // todo: add success/failure callbacks using the provided LoadId.
            match result {
                LoaderResult::Loaded(asset_id, asset, _load_id) => {
                    self.assets.insert(asset_id, asset);
                }
                LoaderResult::Unloaded(asset_id) => {
                    self.assets.remove(&asset_id);
                }
                LoaderResult::LoadError(asset_id, _load_id, error_kind) => {
                    self.load_errors.insert(asset_id, error_kind);
                }
            }
        }
    }

    pub(crate) fn is_err(&self, handle_id: HandleId) -> bool {
        if let Some(asset_id) = self.get_asset_id(handle_id) {
            return self.load_errors.contains_key(&asset_id);
        }
        false
    }
}

#[cfg(test)]
mod tests {

    use std::{thread, time::Duration};

    use legion_content_store::{ContentStore, RamContentStore};

    use crate::{
        manifest::Manifest, test_asset, AssetRegistry, AssetRegistryOptions, Resource, ResourceId,
    };

    fn setup_singular_asset_test(content: &[u8]) -> (ResourceId, AssetRegistry) {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let asset_id = {
            let id = ResourceId::new(test_asset::TestAsset::TYPE, 1);
            let checksum = content_store.store(content).unwrap();
            manifest.insert(id, checksum, content.len());
            id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(content_store, manifest)
            .add_loader::<test_asset::TestAsset>()
            .create();

        (asset_id, reg)
    }

    fn setup_dependency_test() -> (ResourceId, ResourceId, AssetRegistry) {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_parent_assetfile = [
            97, 115, 102, 116, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112,
            97, 114, 101, 110, 116,
        ];
        let binary_child_assetfile = [
            97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
        ];

        let child_id = ResourceId::new(test_asset::TestAsset::TYPE, 1);

        let parent_id = {
            manifest.insert(
                child_id,
                content_store.store(&binary_child_assetfile).unwrap(),
                binary_child_assetfile.len(),
            );
            let checksum = content_store.store(&binary_parent_assetfile).unwrap();
            let id = ResourceId::new(test_asset::TestAsset::TYPE, 2);
            manifest.insert(id, checksum, binary_parent_assetfile.len());
            id
        };

        let reg = AssetRegistryOptions::new()
            .add_device_cas(content_store, manifest)
            .add_loader::<test_asset::TestAsset>()
            .create();

        (parent_id, child_id, reg)
    }

    const BINARY_ASSETFILE: [u8; 39] = [
        97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0,
        5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
    ];

    const BINARY_RAWFILE: [u8; 5] = [99, 104, 105, 108, 100];

    #[test]
    fn load_assetfile() {
        let (asset_id, mut reg) = setup_singular_asset_test(&BINARY_ASSETFILE);

        let internal_id;
        {
            let a = reg.load_untyped(asset_id);
            internal_id = a.id;

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_loaded(&reg) {
                let sleep_time = Duration::from_millis(10);
                thread::sleep(sleep_time);
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

    #[test]
    fn load_rawfile() {
        let (asset_id, mut reg) = setup_singular_asset_test(&BINARY_RAWFILE);

        let internal_id;
        {
            let a = reg.load_untyped(asset_id);
            internal_id = a.id;

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_loaded(&reg) {
                let sleep_time = Duration::from_millis(10);
                thread::sleep(sleep_time);
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

    #[test]
    fn load_error() {
        let (_, mut reg) = setup_singular_asset_test(&BINARY_ASSETFILE);

        let internal_id;
        {
            let a = reg.load_untyped(ResourceId::new(test_asset::TestAsset::TYPE, 7));
            internal_id = a.id;

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && !a.is_err(&reg) {
                let sleep_time = Duration::from_millis(10);
                thread::sleep(sleep_time);
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

    #[test]
    fn load_error_sync() {
        let (_, mut reg) = setup_singular_asset_test(&BINARY_ASSETFILE);

        let internal_id;
        {
            let a = reg.load_untyped_sync(ResourceId::new(test_asset::TestAsset::TYPE, 7));
            internal_id = a.id;

            assert!(!a.is_loaded(&reg));
            assert!(a.is_err(&reg));
            assert!(!reg.is_loaded(internal_id));
        }
        reg.update();
        assert!(!reg.is_loaded(internal_id));
    }

    #[test]
    fn load_dependency() {
        let (parent_id, child_id, mut reg) = setup_dependency_test();

        let parent = reg.load_untyped_sync(parent_id);
        assert!(parent.is_loaded(&reg));

        let child = reg.load_untyped(child_id);
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
}
