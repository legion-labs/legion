#![allow(clippy::type_complexity)]

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    io,
    ops::Deref,
    sync::Arc,
    time::Duration,
};

use byteorder::{LittleEndian, ReadBytesExt};
use flurry::TryInsertError;
use serde::{Deserialize, Serialize};

use crate::{vfs, AssetLoader, HandleUntyped, ReferenceUntyped, ResourceId, ResourceType};

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: (ResourceType, ResourceId),
    secondary: (ResourceType, ResourceId),
}

/// The intermediate output of asset loading process.
///
/// Contains the result of loading a single file.
struct LoadOutput {
    /// None here means the asset was already loaded before so it doesn't have to be
    /// loaded again. It will still contribute to reference count though.
    assets: Vec<(
        (ResourceType, ResourceId),
        Option<Box<dyn Any + Send + Sync>>,
    )>,
    load_dependencies: Vec<AssetReference>,
}

pub(crate) enum LoaderResult {
    Loaded(HandleUntyped, Box<dyn Any + Send + Sync>, Option<LoadId>),
    Unloaded((ResourceType, ResourceId)),
    LoadError(HandleUntyped, Option<LoadId>, io::ErrorKind),
    Reloaded(HandleUntyped, Box<dyn Any + Send + Sync>),
}

pub(crate) enum LoaderRequest {
    Load(HandleUntyped, Option<LoadId>),
    Reload(HandleUntyped),
    Unload((ResourceType, ResourceId)),
    Terminate,
}

/// State of a load request in progress.
struct LoadState {
    primary_handle: HandleUntyped,
    /// If load_id is available it means the load was triggered by the user.
    /// Otherwise it is a load of a dependent Resource.
    load_id: Option<LoadId>,
    /// List of Resources in asset file identified by `primary_id`.
    /// None indicates a skipped secondary resource that was already loaded through another resource file.
    assets: Vec<(HandleUntyped, Option<Box<dyn Any + Send + Sync>>)>,
    /// The list of Resources that need to be loaded before the LoadState can be considered completed.
    references: Vec<HandleUntyped>,
}

struct HandleMap {
    unload_tx: crossbeam_channel::Sender<(ResourceType, ResourceId)>,
    handles: flurry::HashMap<(ResourceType, ResourceId), ReferenceUntyped>,
}

impl HandleMap {
    fn new(unload_tx: crossbeam_channel::Sender<(ResourceType, ResourceId)>) -> Arc<Self> {
        Arc::new(Self {
            unload_tx,
            handles: flurry::HashMap::new(),
        })
    }

    fn create_handle(&self, type_id: (ResourceType, ResourceId)) -> HandleUntyped {
        let handle = HandleUntyped::new_handle(type_id, self.unload_tx.clone());

        let weak_ref = HandleUntyped::downgrade(&handle);
        match self.handles.pin().try_insert(type_id, weak_ref) {
            Ok(_) => handle,
            Err(TryInsertError {
                current,
                not_inserted: _,
            }) => {
                handle.forget();
                current.upgrade().unwrap()
            }
        }
    }
}

impl Deref for HandleMap {
    type Target = flurry::HashMap<(ResourceType, ResourceId), ReferenceUntyped>;

    fn deref(&self) -> &Self::Target {
        &self.handles
    }
}

pub(crate) fn create_loader(
    devices: Vec<Box<dyn vfs::Device>>,
) -> (AssetLoaderStub, AssetLoaderIO) {
    let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
    let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();

    let unload_channel = crossbeam_channel::unbounded();
    let handles = HandleMap::new(unload_channel.0);

    let io = AssetLoaderIO::new(
        devices,
        request_tx.clone(),
        request_rx,
        result_tx,
        handles.clone(),
    );
    let loader = AssetLoaderStub::new(request_tx, result_rx, unload_channel.1, handles);
    (loader, io)
}

pub(crate) struct AssetLoaderStub {
    unload_channel_rx: crossbeam_channel::Receiver<(ResourceType, ResourceId)>,
    handles: Arc<HandleMap>,
    request_tx: crossbeam_channel::Sender<LoaderRequest>,
    result_rx: crossbeam_channel::Receiver<LoaderResult>,
}

type LoadId = u32;

impl AssetLoaderStub {
    fn new(
        request_tx: crossbeam_channel::Sender<LoaderRequest>,
        result_rx: crossbeam_channel::Receiver<LoaderResult>,
        unload_channel_rx: crossbeam_channel::Receiver<(ResourceType, ResourceId)>,
        handles: Arc<HandleMap>,
    ) -> Self {
        Self {
            unload_channel_rx,
            handles,
            request_tx,
            result_rx,
        }
    }

    pub(crate) fn get_handle(&self, type_id: (ResourceType, ResourceId)) -> Option<HandleUntyped> {
        self.handles
            .pin()
            .get(&type_id)
            .and_then(ReferenceUntyped::upgrade)
    }

    pub(crate) fn get_or_create_handle(
        &self,
        type_id: (ResourceType, ResourceId),
    ) -> HandleUntyped {
        if let Some(handle) = self.get_handle(type_id) {
            return handle;
        }

        self.handles.create_handle(type_id)
    }

    pub(crate) fn collect_dropped_handles(&self) -> Vec<(ResourceType, ResourceId)> {
        let mut all_removed = vec![];
        while let Ok(unload_id) = self.unload_channel_rx.try_recv() {
            let handles = self.handles.pin();
            let removed = handles.remove(&unload_id).expect("weak ref");
            assert!(removed.upgrade().is_none(), "a load after unload occurred");
            all_removed.push(unload_id);
        }
        all_removed
    }

    pub(crate) fn terminate(&self) {
        self.request_tx.send(LoaderRequest::Terminate).unwrap();
    }

    pub(crate) fn load(&self, resource_id: (ResourceType, ResourceId)) -> HandleUntyped {
        let handle = self.get_or_create_handle(resource_id);

        // todo: for now, this is a made up number to track the id of the load request
        // as we don't currently have load notifications it doesn't mean much.
        // this would have to be changed in order to add load notifications.
        let load_id = 7;
        self.request_tx
            .send(LoaderRequest::Load(handle.clone(), Some(load_id)))
            .unwrap();
        handle
    }

    pub(crate) fn reload(&self, resource_id: (ResourceType, ResourceId)) -> bool {
        self.get_handle(resource_id).map_or(false, |handle| {
            self.request_tx.send(LoaderRequest::Reload(handle)).unwrap();
            true
        })
    }

    pub(crate) fn try_result(&self) -> Option<LoaderResult> {
        self.result_rx.try_recv().ok()
    }

    pub(crate) fn unload(&self, type_id: (ResourceType, ResourceId)) {
        self.request_tx
            .send(LoaderRequest::Unload(type_id))
            .unwrap();
    }
}

const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";

pub(crate) struct AssetLoaderIO {
    loaders: HashMap<ResourceType, Box<dyn AssetLoader + Send>>,

    handles: Arc<HandleMap>,

    /// List of load requests waiting for all references to be loaded.
    processing_list: Vec<LoadState>,

    loaded_resources: HashSet<(ResourceType, ResourceId)>,

    devices: Vec<Box<dyn vfs::Device>>,

    /// Loopback for load requests.
    request_tx: crossbeam_channel::Sender<LoaderRequest>,

    /// Entry point for load requests.
    request_rx: Option<crossbeam_channel::Receiver<LoaderRequest>>,

    /// Output of loader results.
    result_tx: crossbeam_channel::Sender<LoaderResult>,
}

impl AssetLoaderIO {
    fn new(
        devices: Vec<Box<dyn vfs::Device>>,
        request_tx: crossbeam_channel::Sender<LoaderRequest>,
        request_rx: crossbeam_channel::Receiver<LoaderRequest>,
        result_tx: crossbeam_channel::Sender<LoaderResult>,
        handles: Arc<HandleMap>,
    ) -> Self {
        Self {
            loaders: HashMap::new(),
            handles,
            processing_list: Vec::new(),
            loaded_resources: HashSet::new(),
            devices,
            request_tx,
            request_rx: Some(request_rx),
            result_tx,
        }
    }

    pub(crate) fn register_loader(
        &mut self,
        kind: ResourceType,
        loader: Box<dyn AssetLoader + Send>,
    ) {
        self.loaders.insert(kind, loader);
    }

    fn load_resource(&self, type_id: (ResourceType, ResourceId)) -> io::Result<Vec<u8>> {
        for device in &self.devices {
            if let Some(content) = device.load(type_id) {
                return Ok(content);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Resource Not Found",
        ))
    }

    fn reload_resource(&self, type_id: (ResourceType, ResourceId)) -> io::Result<Vec<u8>> {
        for device in &self.devices {
            if let Some(content) = device.reload(type_id) {
                return Ok(content);
            }
        }

        // fallback to loading existing resources.
        self.load_resource(type_id)
    }

    fn process_reload(&mut self, primary_handle: &HandleUntyped) -> Result<(), io::Error> {
        let primary_id = primary_handle.id();
        let asset_data = self.reload_resource(primary_id)?;

        let load_func = {
            if asset_data.len() < 4 || &asset_data[0..4] != ASSET_FILE_TYPENAME {
                Self::load_raw
            } else {
                Self::load_asset_file
            }
        };

        let mut output = load_func(primary_handle, &mut &asset_data[..], &mut self.loaders)?;

        assert!(
            output
                .load_dependencies
                .iter()
                .all(|reference| self.loaded_resources.contains(&reference.primary)),
            "Loading new dependencies not supported"
        );

        assert_eq!(
            output.assets.len(),
            1,
            "Reload of secondary assets not supported"
        );

        let (_, primary_resource) = output.assets.first_mut().unwrap();

        if let Some(boxed) = primary_resource {
            let loader = self.loaders.get_mut(&primary_id.0).unwrap();
            loader.load_init(boxed.as_mut());
        }
        assert!(self.loaded_resources.contains(&primary_id));

        if let Some(resource) = primary_resource.take() {
            self.result_tx
                .send(LoaderResult::Reloaded(primary_handle.clone(), resource))
                .unwrap();
        }

        Ok(())
    }

    fn process_load(
        &mut self,
        primary_handle: HandleUntyped,
        load_id: Option<u32>,
    ) -> Result<(), (HandleUntyped, Option<LoadId>, io::Error)> {
        let primary_id = primary_handle.id();
        if self.loaded_resources.contains(&primary_id)
            || self
                .processing_list
                .iter()
                .any(|state| state.primary_handle == primary_handle)
        {
            // todo: we should create a LoadState based on existing load state?
            // this way the load result will be notified when the resource is actually loaded.
            return Ok(());
        }
        let asset_data = self
            .load_resource(primary_id)
            .map_err(|e| (primary_handle.clone(), load_id, e))?;

        let load_func = {
            if asset_data.len() < 4 || &asset_data[0..4] != ASSET_FILE_TYPENAME {
                Self::load_raw
            } else {
                Self::load_asset_file
            }
        };

        let output = load_func(&primary_handle, &mut &asset_data[..], &mut self.loaders)
            .map_err(|e| (primary_handle.clone(), load_id, e))?;

        let references = output
            .load_dependencies
            .iter()
            .map(|reference| self.handles.create_handle(reference.primary))
            .collect::<Vec<_>>();

        for reference in &references {
            self.request_tx
                .send(LoaderRequest::Load(reference.clone(), None))
                .unwrap();
        }
        self.processing_list.push(LoadState {
            primary_handle,
            load_id,
            assets: output
                .assets
                .into_iter()
                .map(|(secondary_id, boxed)| {
                    let handle = self.handles.create_handle(secondary_id);
                    (handle, boxed)
                })
                .collect::<Vec<_>>(),
            references,
        });
        Ok(())
    }

    fn process_unload(&mut self, resource_id: (ResourceType, ResourceId)) {
        self.loaded_resources.remove(&resource_id);
        self.result_tx
            .send(LoaderResult::Unloaded(resource_id))
            .unwrap();
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_request(
        &mut self,
        request: LoaderRequest,
    ) -> Result<(), (HandleUntyped, Option<LoadId>, io::Error)> {
        match request {
            LoaderRequest::Load(primary_handle, load_id) => {
                self.process_load(primary_handle, load_id)
            }
            LoaderRequest::Reload(primary_handle) => self
                .process_reload(&primary_handle)
                .map_err(|e| (primary_handle, None, e)),
            LoaderRequest::Unload(resource_id) => {
                self.process_unload(resource_id);
                Ok(())
            }
            LoaderRequest::Terminate => {
                self.request_rx = None;
                Ok(())
            }
        }
    }

    pub(crate) fn wait(&mut self, timeout: Duration) -> Option<usize> {
        // process new pending requests
        let mut errors = vec![];
        loop {
            match &self.request_rx {
                None => return None,
                Some(request_rx) => match request_rx.recv_timeout(timeout) {
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return None,
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => break,
                    Ok(request) => {
                        if let Err(error) = self.process_request(request) {
                            errors.push(error);
                        }
                    }
                },
            }
        }

        // todo: propagate errors to dependent assets before sending results.
        for (load_failed, _, err) in errors {
            let (failed, pending): (Vec<_>, Vec<_>) = std::mem::take(&mut self.processing_list)
                .into_iter()
                .partition(|pending| pending.references.iter().any(|reff| reff == &load_failed));

            for failed_pending in failed {
                self.result_tx
                    .send(LoaderResult::LoadError(
                        failed_pending.primary_handle,
                        failed_pending.load_id,
                        err.kind(),
                    ))
                    .unwrap();
            }
            self.result_tx
                .send(LoaderResult::LoadError(load_failed, None, err.kind()))
                .unwrap();

            self.processing_list = pending;
        }

        // check for completion.
        for index in (0..self.processing_list.len()).rev() {
            let pending = &self.processing_list[index];
            let finished = pending
                .references
                .iter()
                .all(|reference| self.loaded_resources.contains(&reference.id()));
            if finished {
                let mut loaded = self.processing_list.swap_remove(index);

                for (asset_id, asset) in &mut loaded.assets {
                    if let Some(boxed) = asset {
                        let loader = self.loaders.get_mut(&asset_id.id().0).unwrap();
                        loader.load_init(boxed.as_mut());
                    }
                    // if there is no boxed asset here, it means it was already loaded before.
                }

                for (handle, _) in &loaded.assets {
                    self.loaded_resources.insert(handle.id());
                }

                // send primary asset with load_id. all secondary assets without to not cause load notification.
                let mut asset_iter = loaded.assets.into_iter();
                let primary_asset = asset_iter.next().unwrap().1.unwrap();
                self.result_tx
                    .send(LoaderResult::Loaded(
                        loaded.primary_handle,
                        primary_asset,
                        loaded.load_id,
                    ))
                    .unwrap();

                for (id, asset) in asset_iter {
                    if let Some(asset) = asset {
                        self.result_tx
                            .send(LoaderResult::Loaded(id, asset, None))
                            .unwrap();
                    }
                }
            }
        }

        Some(self.processing_list.len())
    }

    fn load_raw(
        handle: &HandleUntyped,
        reader: &mut dyn io::Read,
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send>>,
    ) -> Result<LoadOutput, io::Error> {
        let type_id = handle.id();
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;

        let asset_type = type_id.0;
        let loader = loaders.get_mut(&asset_type).unwrap();
        let boxed_asset = loader.load(&mut &content[..])?;

        Ok(LoadOutput {
            assets: vec![(type_id, Some(boxed_asset))],
            load_dependencies: vec![],
        })
    }

    fn load_asset_file(
        primary_handle: &HandleUntyped,
        reader: &mut dyn io::Read,
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send>>,
    ) -> Result<LoadOutput, io::Error> {
        let primary_id = primary_handle.id();
        const ASSET_FILE_VERSION: u16 = 1;

        let mut typename: [u8; 4] = [0; 4];
        reader.read_exact(&mut typename)?;
        if &typename != ASSET_FILE_TYPENAME {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Filetype Mismatch",
            ));
        }

        // asset file header
        let version = reader.read_u16::<LittleEndian>()?;
        if version != ASSET_FILE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Version Mismatch",
            ));
        }

        let reference_count = reader.read_u64::<LittleEndian>()?;
        let mut reference_list = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            let asset_ref = (
                ResourceType::from_raw(reader.read_u32::<LittleEndian>()?),
                ResourceId::from_raw(reader.read_u128::<LittleEndian>()?),
            );
            reference_list.push(AssetReference {
                primary: asset_ref,
                secondary: asset_ref,
            });
        }

        // section header
        let asset_type = unsafe {
            std::mem::transmute::<u32, ResourceType>(
                reader.read_u32::<LittleEndian>().expect("valid data"),
            )
        };
        assert_eq!(
            asset_type, primary_id.0,
            "The asset must be of primary id's type"
        );

        let asset_count = reader.read_u64::<LittleEndian>().expect("valid data");
        assert_eq!(
            asset_count, 1,
            "For now, only 1 asset - the primary asset - is expected"
        );

        let nbytes = reader.read_u64::<LittleEndian>().expect("valid data");

        let mut content = Vec::new();
        content.resize(nbytes as usize, 0);
        reader.read_exact(&mut content).expect("valid data");

        let loader = loaders.get_mut(&asset_type).unwrap();
        let boxed_asset = loader.load(&mut &content[..])?;

        // todo: Do not load what was loaded in another primary-asset.
        //
        // There are two cases to consider:
        //
        // Non-reload-case: for *secondary assets* make sure that we only load them if
        // they are not already loaded.
        //
        // `let is_loaded = self.asset_refcounts.contains_key(&secondary_id));`
        //
        // Reload-case: all *secondary assets* should be loaded again.

        Ok(LoadOutput {
            assets: vec![(primary_id, Some(boxed_asset))],
            load_dependencies: reference_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use legion_content_store::{ContentStore, RamContentStore};

    use super::{create_loader, AssetLoaderIO, AssetLoaderStub};
    use crate::{
        asset_loader::{HandleMap, LoaderRequest, LoaderResult},
        manifest::Manifest,
        test_asset, vfs, Handle, Resource, ResourceId, ResourceType,
    };

    fn setup_test() -> ((ResourceType, ResourceId), AssetLoaderStub, AssetLoaderIO) {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let asset_id = {
            let id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(1));
            let checksum = content_store
                .store(&test_asset::tests::BINARY_ASSETFILE)
                .unwrap();
            manifest.insert(id, checksum, test_asset::tests::BINARY_ASSETFILE.len());
            id
        };

        let (loader, mut io) =
            create_loader(vec![Box::new(vfs::CasDevice::new(manifest, content_store))]);
        io.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        (asset_id, loader, io)
    }

    #[test]
    fn typed_ref() {
        let (asset_id, loader, mut io) = setup_test();

        {
            let untyped = loader.load(asset_id);

            assert_eq!(untyped.id(), asset_id);

            let typed: Handle<test_asset::TestAsset> = untyped.into();

            io.wait(Duration::from_millis(500));
            assert!(loader.handles.pin().get(&typed.id()).is_some());

            match loader.try_result() {
                Some(LoaderResult::Loaded(handle, _, _)) => {
                    assert!(handle.id() == typed.id());
                }
                _ => panic!(),
            }
        }

        loader.collect_dropped_handles();

        assert!(!loader.handles.pin().contains_key(&asset_id));

        let typed: Handle<test_asset::TestAsset> = loader.load(asset_id).into();
        io.wait(Duration::from_millis(500));

        assert!(loader.handles.pin().get(&typed.id()).is_some());
    }

    #[test]
    fn call_load_twice() {
        let (asset_id, loader, _io) = setup_test();

        let a = loader.load(asset_id);
        {
            let b = a.clone();
            assert_eq!(a.id(), b.id());
            {
                let c = loader.load(asset_id);
                assert_eq!(a.id(), c.id());
            }
        }
    }

    #[test]
    fn load_no_dependencies() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let asset_id = {
            let id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(1));
            let checksum = content_store
                .store(&test_asset::tests::BINARY_ASSETFILE)
                .unwrap();
            manifest.insert(id, checksum, test_asset::tests::BINARY_ASSETFILE.len());
            id
        };

        let (unload_tx, _unload_rx) = crossbeam_channel::unbounded::<_>();

        let handles = HandleMap::new(unload_tx);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle, load_id))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&asset_id));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }

    #[test]
    fn load_failed_dependency() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let parent_id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(2));

        let asset_id = {
            let checksum = content_store
                .store(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .unwrap();
            manifest.insert(
                parent_id,
                checksum,
                test_asset::tests::BINARY_PARENT_ASSETFILE.len(),
            );
            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle, load_id))
            .expect("valid tx");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::LoadError(_, _, _)));
    }

    #[test]
    fn load_with_dependency() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let parent_content = "parent";

        let parent_id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(2));
        let child_id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(1));

        let asset_id = {
            manifest.insert(
                child_id,
                content_store
                    .store(&test_asset::tests::BINARY_ASSETFILE)
                    .unwrap(),
                test_asset::tests::BINARY_ASSETFILE.len(),
            );
            let checksum = content_store
                .store(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .unwrap();
            manifest.insert(
                parent_id,
                checksum,
                test_asset::tests::BINARY_PARENT_ASSETFILE.len(),
            );

            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle.clone(), load_id))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&parent_id));

        let mut result = None;
        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        if let Ok(res) = result_rx.try_recv() {
            // child load result comes first with no load_id..
            assert!(matches!(res, LoaderResult::Loaded(_, _, None)));
        }

        if let Ok(res) = result_rx.try_recv() {
            // ..followed by parent load result with load_id
            assert!(matches!(res, LoaderResult::Loaded(_, _, Some(_))));
            result = Some(res);
        }

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(matches!(result, LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&parent_id));
        assert!(loader.loaded_resources.contains(&child_id));

        if let LoaderResult::Loaded(handle, asset, returned_load_id) = result {
            let asset = asset.downcast_ref::<test_asset::TestAsset>().unwrap();
            assert_eq!(asset.content, parent_content);
            assert_eq!(asset_handle, handle);
            assert_eq!(returned_load_id, load_id);
        }

        // unload and validate references.

        request_tx
            .send(LoaderRequest::Unload(parent_id))
            .expect("to send request");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&parent_id));

        /*
            assert_eq!(result.assets.len(), 1);
            assert_eq!(result._load_dependencies.len(), 1);

            let (asset_id, asset) = &result.assets[0];

            let asset = asset.downcast_ref::<TestAsset>().unwrap();
            assert_eq!(asset.content, expected_content);
            assert_eq!(asset_id, &id);
        */
    }

    #[test]
    fn reload_no_dependencies() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let asset_id = {
            let id = (test_asset::TestAsset::TYPE, ResourceId::new_explicit(1));
            let checksum = content_store
                .store(&test_asset::tests::BINARY_ASSETFILE)
                .unwrap();
            manifest.insert(id, checksum, test_asset::tests::BINARY_ASSETFILE.len());
            id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        request_tx
            .send(LoaderRequest::Load(asset_handle.clone(), None))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&asset_id));

        // reload
        request_tx
            .send(LoaderRequest::Reload(asset_handle))
            .unwrap();

        let mut result = None;
        loader.wait(Duration::from_millis(10));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Reloaded(_, _)));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }
}
