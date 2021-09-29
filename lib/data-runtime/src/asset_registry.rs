use std::{
    collections::HashMap,
    io,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use legion_content_store::ContentStore;

use crate::{
    asset_loader::{create_loader, AssetLoaderStub, LoaderResult},
    manifest::Manifest,
    Asset, AssetId, AssetLoader, AssetType, Handle, HandleId, HandleUntyped, RefOp,
};

/// Options which can be used to configure the creation of [`AssetRegistry`].
pub struct AssetRegistryOptions {
    creators: HashMap<AssetType, Box<dyn AssetLoader + Send>>,
}

impl AssetRegistryOptions {
    /// Creates a blank set of options for [`AssetRegistry`] configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
        }
    }

    /// Enables support of a given [`Asset`] by adding corresponding [`AssetLoader`].
    pub fn add_creator(
        mut self,
        asset_kind: AssetType,
        creator: Box<dyn AssetLoader + Send>,
    ) -> Self {
        self.creators.insert(asset_kind, creator);
        self
    }

    /// Creates [`AssetRegistry`] based on `AssetRegistryOptions`.
    pub fn create(self, content_store: Box<dyn ContentStore>, manifest: Manifest) -> AssetRegistry {
        let (loader, mut io) = create_loader(content_store, manifest);

        for (kind, creator) in self.creators {
            io.register_creator(kind, creator);
        }

        let load_thread = thread::spawn(move || {
            let mut loader = io;
            while loader.wait(Duration::from_millis(100)).is_some() {}
        });

        AssetRegistry {
            id_generator: 0,
            refcount_channel: crossbeam_channel::unbounded(),
            ref_counts: HashMap::new(),
            assets: HashMap::new(),
            load_errors: HashMap::new(),
            load_thread: Some(load_thread),
            loader,
        }
    }
}

/// Registry of all loaded [`Asset`]s.
///
/// Provides an API to load assets by their [`AssetId`]. The lifetime of an [`Asset`] is determined
/// by the reference counted [`HandleUntyped`] and [`Handle`].
///
/// [`Handle`]: [`crate::Handle`]
pub struct AssetRegistry {
    id_generator: HandleId,
    refcount_channel: (
        crossbeam_channel::Sender<RefOp>,
        crossbeam_channel::Receiver<RefOp>,
    ),
    ref_counts: HashMap<HandleId, (AssetId, isize)>,
    assets: HashMap<AssetId, Arc<dyn Asset + Send + Sync>>,
    load_errors: HashMap<AssetId, io::ErrorKind>,
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
    pub fn load_untyped(&mut self, id: AssetId) -> HandleUntyped {
        let handle = self.create_handle(id);
        self.loader.load(id, handle.id);
        handle
    }

    /// Same as [`Self::load_untyped`] but the returned handle is generic over asset type `T` for convenience.
    pub fn load<T: Asset>(&mut self, id: AssetId) -> Handle<T> {
        let handle = self.load_untyped(id);
        Handle::<T>::from(handle)
    }

    /// Retrieves the asset id associated with a handle.
    pub(crate) fn get_asset_id(&self, handle_id: HandleId) -> Option<AssetId> {
        self.ref_counts
            .get(&handle_id)
            .map(|(asset_id, _)| *asset_id)
    }

    /// Retrieves a reference to an asset, None if asset is not loaded.
    pub(crate) fn get<T: Asset>(&self, handle_id: HandleId) -> Option<&T> {
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
        self.process_refcount_ops();

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

    fn process_refcount_ops(&mut self) {
        while let Ok(op) = self.refcount_channel.1.try_recv() {
            match op {
                RefOp::AddRef(id) => {
                    let (_, count) = self.ref_counts.get_mut(&id).unwrap();
                    *count += 1;
                }
                RefOp::RemoveRef(id) => {
                    let (_, count) = self.ref_counts.get_mut(&id).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        self.remove_handle(id);
                    }
                }
            }
        }
    }

    fn create_handle(&mut self, id: AssetId) -> HandleUntyped {
        self.id_generator += 1;
        let new_id = self.id_generator;
        // insert data
        self.ref_counts.insert(new_id, (id, 1));
        HandleUntyped::create(new_id, self.refcount_channel.0.clone())
    }

    fn remove_handle(&mut self, handle_id: HandleId) {
        // remove data
        if let Some((removed_id, rc)) = self.ref_counts.remove(&handle_id) {
            self.load_errors.remove(&removed_id);
            self.assets.remove(&removed_id);
            self.loader.unload(removed_id);
            assert_eq!(rc, 0);
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
        manifest::Manifest, test_asset, AssetDescriptor, AssetId, AssetRegistry,
        AssetRegistryOptions, Handle,
    };

    fn setup_test() -> (AssetId, AssetRegistry) {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_assetfile = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 181, 1, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0,
            0, 0, 0, 116, 101, 115, 116, 95, 99, 111, 110, 116, 101, 110, 116,
        ];

        let asset_id = {
            let id = AssetId::new(test_asset::TestAsset::TYPE, 1);
            let checksum = content_store.store(&binary_assetfile).unwrap();
            manifest.insert(id, checksum.into(), binary_assetfile.len());
            id
        };

        let reg = AssetRegistryOptions::new()
            .add_creator(
                test_asset::TestAsset::TYPE,
                Box::new(test_asset::TestAssetCreator {}),
            )
            .create(content_store, manifest);

        (asset_id, reg)
    }

    #[test]
    fn ref_count() {
        let (asset_id, mut reg) = setup_test();

        let internal_id;
        {
            let a = reg.load_untyped(asset_id);
            internal_id = a.id;
            assert_eq!(reg.ref_counts.get(&a.id).unwrap().1, 1);

            {
                let b = a.clone();
                reg.process_refcount_ops();

                assert_eq!(reg.ref_counts.get(&b.id).unwrap().1, 2);
                assert_eq!(reg.ref_counts.get(&a.id).unwrap().1, 2);
                assert_eq!(a, b);
            }
            reg.process_refcount_ops();
            assert_eq!(reg.ref_counts.get(&a.id).unwrap().1, 1);
        }
        reg.process_refcount_ops();
        assert!(!reg.ref_counts.contains_key(&internal_id));
    }

    #[test]
    fn typed_ref() {
        let (asset_id, mut reg) = setup_test();

        let internal_id;
        {
            let untyped = reg.load_untyped(asset_id);
            assert_eq!(reg.ref_counts.get(&untyped.id).unwrap().1, 1);

            internal_id = untyped.id;

            let typed: Handle<test_asset::TestAsset> = untyped.into();
            reg.update();
            assert_eq!(reg.ref_counts.get(&typed.id).unwrap().1, 1);

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && typed.get(&reg).is_none() {
                let sleep_time = Duration::from_millis(10);
                thread::sleep(sleep_time);
                test_timeout -= sleep_time;
                reg.update();
            }
            assert!(typed.get(&reg).is_some());
        }

        reg.update(); // to drop the refcount to zero.

        assert!(!reg.ref_counts.contains_key(&internal_id));

        let typed = reg.load::<test_asset::TestAsset>(asset_id);

        let mut test_timeout = Duration::from_millis(500);
        while test_timeout > Duration::ZERO && typed.get(&reg).is_none() {
            let sleep_time = Duration::from_millis(10);
            thread::sleep(sleep_time);
            test_timeout -= sleep_time;
            reg.update();
        }
        assert!(typed.get(&reg).is_some());
    }
}
