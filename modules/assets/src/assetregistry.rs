use crate::{
    assetloader::{AssetLoader, AssetLoaderStorage},
    handle::{AssetGenericHandle, AssetHandleId, AssetRefCounter},
    Asset, AssetCreator, AssetId, AssetType,
};

use std::{
    collections::HashMap,
    io, mem,
    sync::{Arc, Mutex},
};

enum AssetState {
    Loaded(Box<dyn Asset>),
    Loading,
    LoadFailed(io::Error),
}

/// Reference counted storage of assets.
struct AssetStorage {
    refs: Arc<Mutex<AssetRefCounter>>,
    assets: HashMap<AssetHandleId, AssetState>,
    handles: HashMap<AssetId, AssetHandleId>,
    handle_generator: AssetHandleId,
}

enum HandleResult {
    New(AssetGenericHandle),
    Existing(AssetGenericHandle),
}

impl AssetStorage {
    pub fn new() -> Self {
        Self {
            refs: Arc::new(Mutex::new(AssetRefCounter::new())),
            assets: HashMap::new(),
            handles: HashMap::new(),
            handle_generator: AssetHandleId(1),
        }
    }

    fn create_handle(&mut self, id: AssetId) -> AssetGenericHandle {
        let handle_id = {
            let handle = self.handle_generator;
            self.handle_generator = AssetHandleId(self.handle_generator.0 + 1);
            handle
        };

        let handle = AssetGenericHandle::new(self.refs.clone(), handle_id);
        self.handles.insert(id, handle.handle_id);
        self.assets.insert(handle.handle_id, AssetState::Loading);
        handle
    }

    /// Returns a copy of an asset's handle if one exists, None otherwise.
    pub fn find(&self, id: AssetId) -> Option<AssetGenericHandle> {
        let handle_id = self.handles.get(&id)?;
        Some(AssetGenericHandle::new(self.refs.clone(), *handle_id))
    }

    /// Releases assets that are no longer refered externally and internally.
    pub fn collect_garbage(&mut self) {
        for orphan in mem::take(&mut self.refs.lock().unwrap().orphans()) {
            let id = *self
                .handles
                .iter()
                .find_map(|(key, &value)| if value == orphan { Some(key) } else { None })
                .unwrap();

            self.handles.remove(&id);
            self.assets.remove(&orphan);
        }
    }

    pub fn get_or_create_handle(&mut self, id: AssetId) -> HandleResult {
        if let Some(handle) = self.find(id) {
            HandleResult::Existing(handle)
        } else {
            HandleResult::New(self.create_handle(id))
        }
    }

    fn get<A: Asset>(&self, handle: &AssetGenericHandle) -> Option<&A> {
        if let AssetState::Loaded(asset) = self.assets.get(&handle.handle_id)? {
            asset.as_ref().as_any().downcast_ref::<A>()
        } else {
            None
        }
    }
}

/// An interface to load runtime assets.
///
/// Handles user-defined types across a variaty of libraries.
pub struct AssetRegistry {
    references: AssetStorage,
    loader: AssetLoader,
}

impl AssetLoaderStorage for AssetStorage {
    fn store(&mut self, id: AssetId, asset: Result<Box<dyn Asset>, io::Error>) {
        if let Some(handle_id) = self.handles.get(&id) {
            let asset_state = self.assets.get_mut(handle_id).unwrap();

            if let AssetState::Loading = asset_state {
                match asset {
                    Ok(asset) => {
                        *asset_state = AssetState::Loaded(asset);
                    }
                    Err(err) => *asset_state = AssetState::LoadFailed(err),
                }
            } else {
                panic!("trying to store asset twice?");
            }
        } else {
            // no more ref-counts. dropping the asset.
        }
    }
}

impl Default for AssetRegistry {
    /// Creates a new `AssetRegistry` instance.
    fn default() -> Self {
        Self {
            references: AssetStorage::new(),
            loader: AssetLoader::new(),
        }
    }
}

impl AssetRegistry {
    /// Register a creator for an asset type.
    ///
    /// [`AssetCreator`] is responsible for asset loading: memory management, deserialization, post-load initialization.
    pub fn register_type(&mut self, kind: AssetType, creator: Box<dyn AssetCreator>) {
        self.loader.register_creator(kind, creator);
    }

    /// Requests asset loading.
    ///
    /// This method will look for a primary asset of `id` and try to load it.
    pub fn load(&mut self, id: AssetId) -> Result<AssetGenericHandle, io::Error> {
        let handle = match self.references.get_or_create_handle(id) {
            HandleResult::New(handle) => {
                self.loader.load_request(handle.clone(), id);
                handle
            }
            HandleResult::Existing(handle) => handle,
        };
        Ok(handle)
    }

    /// Processes asset loading queue.
    ///
    /// todo(kstasik): Move this work to an IO thread.
    pub fn loader_update(&mut self) {
        self.loader.load_update(&mut self.references);
        self.references.collect_garbage();
    }

    /// Return a reference to an assset behind a given handle.
    ///
    /// This method will return None if the asset is not loaded.
    pub fn get<A: Asset>(&self, handle: &AssetGenericHandle) -> Option<&A> {
        self.references.get(handle)
    }
}

#[cfg(test)]
mod tests {

    use std::any::Any;

    use crate::{
        assetloader::AssetLoaderStorage, assetregistry::AssetStorage, test_asset, AssetId,
    };

    use super::Asset;

    struct SampleAsset {}
    struct DummyAsset {}

    impl Asset for SampleAsset {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    impl Asset for DummyAsset {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn asset_handle() {
        let mut reg = AssetStorage::new();

        let id = AssetId::new(test_asset::TYPE_ID, 1);
        let asset = Box::new(SampleAsset {});

        assert_eq!(reg.find(id), None);

        {
            let _outer_handle = {
                let handle = reg.create_handle(id);

                // asset is there but not yet loaded.
                assert_eq!(reg.find(id).as_ref(), Some(&handle));
                assert!(reg.get::<SampleAsset>(&handle).is_none());

                // here, asset loading finishes.
                reg.store(id, Ok(asset));

                assert_eq!(reg.find(id).as_ref(), Some(&handle));
                assert!(reg.get::<SampleAsset>(&handle).is_some());
                assert!(reg.get::<DummyAsset>(&handle).is_none());

                // `handle` holds the reference.
                reg.collect_garbage();
                assert_ne!(reg.find(id), None);

                handle
            };

            // `outer_handle` still holds the reference.
            reg.collect_garbage();
            assert_ne!(reg.find(id), None);
        }

        // `outer_handle` fell out of scope here therefore the asset should be released.
        reg.collect_garbage();
        assert_eq!(reg.find(id), None);
    }
}
