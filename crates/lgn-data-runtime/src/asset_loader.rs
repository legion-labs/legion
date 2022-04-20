#![allow(clippy::type_complexity)]

use std::{
    collections::{HashMap, HashSet},
    io,
    ops::Deref,
    sync::Arc,
    time::Duration,
};

use byteorder::{LittleEndian, ReadBytesExt};
use flurry::TryInsertError;
use lgn_content_store::ChunkIdentifier;
use lgn_tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    vfs, AssetLoader, AssetRegistryError, HandleUntyped, ReferenceUntyped, Resource, ResourceId,
    ResourceType, ResourceTypeAndId,
};

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: ResourceTypeAndId,
    secondary: ResourceTypeAndId,
}

/// The intermediate output of asset loading process.
///
/// Contains the result of loading a single file.
struct LoadOutput {
    /// None here means the asset was already loaded before so it doesn't have
    /// to be loaded again. It will still contribute to reference count
    /// though.
    assets: Vec<(ResourceTypeAndId, Option<Box<dyn Resource>>)>,
    load_dependencies: Vec<AssetReference>,
}

pub(crate) enum LoaderResult {
    Loaded(HandleUntyped, Box<dyn Resource>, Option<LoadId>),
    Unloaded(ResourceTypeAndId),
    LoadError(HandleUntyped, Option<LoadId>, AssetRegistryError),
    Reloaded(HandleUntyped, Box<dyn Resource>),
}

#[derive(Debug)]
pub(crate) enum LoaderRequest {
    Load(HandleUntyped, Option<LoadId>),
    Reload(HandleUntyped),
    Unload(ResourceTypeAndId),
    Terminate,
    LoadManifest(ChunkIdentifier),
}

/// State of a load request in progress.
struct LoadState {
    primary_handle: HandleUntyped,
    /// If load_id is available it means the load was triggered by the user.
    /// Otherwise it is a load of a dependent Resource.
    load_id: Option<LoadId>,
    /// List of Resources in asset file identified by `primary_id`.
    /// None indicates a skipped secondary resource that was already loaded
    /// through another resource file.
    assets: Vec<(HandleUntyped, Option<Box<dyn Resource>>)>,
    /// The list of Resources that need to be loaded before the LoadState can be
    /// considered completed.
    references: Vec<HandleUntyped>,
    /// Specify if it's a reload
    reload: bool,
}

struct HandleMap {
    unload_tx: crossbeam_channel::Sender<ResourceTypeAndId>,
    handles: flurry::HashMap<ResourceTypeAndId, ReferenceUntyped>,
}

impl HandleMap {
    fn new(unload_tx: crossbeam_channel::Sender<ResourceTypeAndId>) -> Arc<Self> {
        Arc::new(Self {
            unload_tx,
            handles: flurry::HashMap::new(),
        })
    }

    fn create_handle(&self, type_id: ResourceTypeAndId) -> HandleUntyped {
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
    type Target = flurry::HashMap<ResourceTypeAndId, ReferenceUntyped>;

    fn deref(&self) -> &Self::Target {
        &self.handles
    }
}

pub(crate) fn create_loader(
    devices: Vec<Box<(dyn vfs::Device + Send)>>,
) -> (AssetLoaderStub, AssetLoaderIO) {
    let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
    let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();

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
    unload_channel_rx: crossbeam_channel::Receiver<ResourceTypeAndId>,
    handles: Arc<HandleMap>,
    request_tx: tokio::sync::mpsc::UnboundedSender<LoaderRequest>,
    result_rx: crossbeam_channel::Receiver<LoaderResult>,
}

type LoadId = u32;

impl AssetLoaderStub {
    fn new(
        request_tx: tokio::sync::mpsc::UnboundedSender<LoaderRequest>,
        result_rx: crossbeam_channel::Receiver<LoaderResult>,
        unload_channel_rx: crossbeam_channel::Receiver<ResourceTypeAndId>,
        handles: Arc<HandleMap>,
    ) -> Self {
        Self {
            unload_channel_rx,
            handles,
            request_tx,
            result_rx,
        }
    }

    pub(crate) fn get_handle(&self, type_id: ResourceTypeAndId) -> Option<HandleUntyped> {
        self.handles
            .pin()
            .get(&type_id)
            .and_then(ReferenceUntyped::upgrade)
    }

    pub(crate) fn get_or_create_handle(&self, type_id: ResourceTypeAndId) -> HandleUntyped {
        if let Some(handle) = self.get_handle(type_id) {
            return handle;
        }

        self.handles.create_handle(type_id)
    }

    pub(crate) fn collect_dropped_handles(&self) -> Vec<ResourceTypeAndId> {
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
        if let Err(err) = self.request_tx.send(LoaderRequest::Terminate) {
            lgn_tracing::warn!("Failed to terminate AssetLoader: {}", err);
        }
    }

    pub(crate) fn load(&self, resource_id: ResourceTypeAndId) -> HandleUntyped {
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

    pub(crate) fn reload(&self, resource_id: ResourceTypeAndId) -> bool {
        self.get_handle(resource_id).map_or(false, |handle| {
            self.request_tx.send(LoaderRequest::Reload(handle)).unwrap();
            true
        })
    }

    pub(crate) fn try_result(&self) -> Option<LoaderResult> {
        self.result_rx.try_recv().ok()
    }

    pub(crate) fn unload(&self, type_id: ResourceTypeAndId) {
        self.request_tx
            .send(LoaderRequest::Unload(type_id))
            .unwrap();
    }

    pub(crate) fn load_manifest(&self, manifest_id: &ChunkIdentifier) {
        self.request_tx
            .send(LoaderRequest::LoadManifest(manifest_id.clone()))
            .unwrap();
    }
}

const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";

pub(crate) struct AssetLoaderIO {
    loaders: HashMap<ResourceType, Box<dyn AssetLoader + Send + Sync>>,

    handles: Arc<HandleMap>,

    /// List of load requests waiting for all references to be loaded.
    processing_list: Vec<LoadState>,

    loaded_resources: HashSet<ResourceTypeAndId>,

    devices: Vec<Box<(dyn vfs::Device + Send)>>,

    /// Loopback for load requests.
    request_tx: tokio::sync::mpsc::UnboundedSender<LoaderRequest>,

    /// Entry point for load requests.
    request_rx: Option<tokio::sync::mpsc::UnboundedReceiver<LoaderRequest>>,

    /// Output of loader results.
    result_tx: crossbeam_channel::Sender<LoaderResult>,
}

impl AssetLoaderIO {
    fn new(
        devices: Vec<Box<(dyn vfs::Device + Send)>>,
        request_tx: tokio::sync::mpsc::UnboundedSender<LoaderRequest>,
        request_rx: tokio::sync::mpsc::UnboundedReceiver<LoaderRequest>,
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
        loader: Box<dyn AssetLoader + Send + Sync>,
    ) {
        self.loaders.insert(kind, loader);
    }

    async fn load_resource(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<Vec<u8>, AssetRegistryError> {
        let start = std::time::Instant::now();

        for device in &self.devices {
            let res = device.load(type_id).await;
            if let Some(content) = res {
                info!(
                    "Loaded {:?} {} in {:?}",
                    type_id,
                    content.len(),
                    start.elapsed(),
                );
                return Ok(content);
            }
        }
        Err(AssetRegistryError::ResourceNotFound(type_id))
    }

    async fn reload_resource(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<Vec<u8>, AssetRegistryError> {
        for device in &self.devices {
            if let Some(content) = device.reload(type_id).await {
                return Ok(content);
            }
        }

        // fallback to loading existing resources.
        self.load_resource(type_id).await
    }

    async fn process_reload(
        &mut self,
        primary_handle: &HandleUntyped,
    ) -> Result<(), AssetRegistryError> {
        let primary_id = primary_handle.id();
        let asset_data = self.reload_resource(primary_id).await?;

        let load_func = {
            if asset_data.len() < 4 || &asset_data[0..4] != ASSET_FILE_TYPENAME {
                Self::load_raw
            } else {
                Self::load_asset_file
            }
        };

        let output =
            load_func(primary_handle, &mut &asset_data[..], &mut self.loaders).map_err(|err| {
                error!("Error loading {:?}: {}", primary_handle.id(), err);
                err
            })?;

        let references = output
            .load_dependencies
            .iter()
            .filter(|reference| !self.loaded_resources.contains(&reference.primary))
            .map(|reference| self.handles.create_handle(reference.primary))
            .collect::<Vec<_>>();

        for reference in &references {
            self.request_tx
                .send(LoaderRequest::Reload(reference.clone()))
                .unwrap();
        }

        self.processing_list.push(LoadState {
            primary_handle: primary_handle.clone(),
            load_id: None,
            assets: output
                .assets
                .into_iter()
                .map(|(secondary_id, boxed)| {
                    let handle = self.handles.create_handle(secondary_id);
                    (handle, boxed)
                })
                .collect::<Vec<_>>(),
            references,
            reload: self.loaded_resources.contains(&primary_id),
        });

        Ok(())
    }

    async fn process_load(
        &mut self,
        primary_handle: HandleUntyped,
        load_id: Option<u32>,
    ) -> Result<(), (HandleUntyped, Option<LoadId>, AssetRegistryError)> {
        let primary_id = primary_handle.id();

        if self.loaded_resources.contains(&primary_id)
            || self
                .processing_list
                .iter()
                .any(|state| state.primary_handle == primary_handle)
        {
            // todo: we should create a LoadState based on existing load state?
            // this way the load result will be notified when the resource is actually
            // loaded.
            return Ok(());
        }
        let asset_data = self
            .load_resource(primary_id)
            .await
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
            reload: false,
        });
        Ok(())
    }

    fn process_unload(&mut self, resource_id: ResourceTypeAndId) {
        self.loaded_resources.remove(&resource_id);
        self.result_tx
            .send(LoaderResult::Unloaded(resource_id))
            .unwrap();
    }

    async fn process_load_manifest(&mut self, manifest_id: &ChunkIdentifier) {
        for device in &mut self.devices {
            device.reload_manifest(manifest_id).await;
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn process_request(
        &mut self,
        request: LoaderRequest,
    ) -> Result<(), (HandleUntyped, Option<LoadId>, AssetRegistryError)> {
        match request {
            LoaderRequest::Load(primary_handle, load_id) => {
                self.process_load(primary_handle, load_id).await
            }
            LoaderRequest::Reload(primary_handle) => self
                .process_reload(&primary_handle)
                .await
                .map_err(|e| (primary_handle, None, e)),
            LoaderRequest::Unload(resource_id) => {
                self.process_unload(resource_id);
                Ok(())
            }
            LoaderRequest::Terminate => {
                self.request_rx = None;
                Ok(())
            }
            LoaderRequest::LoadManifest(manifest_id) => {
                self.process_load_manifest(&manifest_id).await;
                Ok(())
            }
        }
    }

    pub(crate) async fn wait(&mut self, timeout: Duration) -> Option<usize> {
        // process new pending requests
        let mut errors = vec![];
        loop {
            match &mut self.request_rx {
                None => return None,
                Some(request_rx) => match tokio::time::timeout(timeout, request_rx.recv()).await {
                    Ok(None) => return None, // disconnected
                    Ok(Some(request)) => {
                        if let Err(error) = self.process_request(request).await {
                            errors.push(error);
                        }
                    }
                    Err(_) => break,
                },
            }
        }

        // todo: propagate errors to dependent assets before sending results.
        for (load_failed, load_id, err) in errors {
            let resource_id = load_failed.id();
            self.result_tx
                .send(LoaderResult::LoadError(load_failed, load_id, err))
                .unwrap();

            self.processing_list.iter_mut().for_each(|load_state| {
                load_state
                    .references
                    .retain(|handle| handle.id() != resource_id);
            });
        }

        // check for completion.
        for index in (0..self.processing_list.len()).rev() {
            let pending = &self.processing_list[index];
            let is_reload = pending.reload;
            let finished = pending
                .references
                .iter()
                .all(|reference| self.loaded_resources.contains(&reference.id()));
            if finished {
                let mut loaded = self.processing_list.swap_remove(index);

                for (asset_id, asset) in &mut loaded.assets {
                    if let Some(boxed) = asset {
                        let loader = self.loaders.get_mut(&asset_id.id().kind).unwrap();
                        loader.load_init(boxed.as_mut());
                    }
                    // if there is no boxed asset here, it means it was already
                    // loaded before.
                }

                for (handle, _) in &loaded.assets {
                    self.loaded_resources.insert(handle.id());
                }

                // send primary asset with load_id. all secondary assets without to not cause
                // load notification.
                let mut asset_iter = loaded.assets.into_iter();
                let primary_asset = asset_iter.next().unwrap().1.unwrap();
                if is_reload {
                    self.result_tx
                        .send(LoaderResult::Reloaded(loaded.primary_handle, primary_asset))
                        .unwrap();
                } else {
                    self.result_tx
                        .send(LoaderResult::Loaded(
                            loaded.primary_handle,
                            primary_asset,
                            loaded.load_id,
                        ))
                        .unwrap();
                }

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
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send + Sync>>,
    ) -> Result<LoadOutput, AssetRegistryError> {
        let type_id = handle.id();
        let mut content = Vec::new();
        reader
            .read_to_end(&mut content)
            .map_err(|err| AssetRegistryError::ResourceIOError(type_id, err))?;

        let asset_type = type_id.kind;
        let loader = loaders
            .get_mut(&asset_type)
            .ok_or(AssetRegistryError::AssetLoaderNotFound(asset_type))?;

        let boxed_asset = loader
            .load(&mut &content[..])
            .map_err(|err| AssetRegistryError::AssetLoaderFailed(type_id, err))?;

        Ok(LoadOutput {
            assets: vec![(type_id, Some(boxed_asset))],
            load_dependencies: vec![],
        })
    }

    fn load_asset_file(
        primary_handle: &HandleUntyped,
        reader: &mut dyn io::Read,
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send + Sync>>,
    ) -> Result<LoadOutput, AssetRegistryError> {
        let primary_id = primary_handle.id();
        const ASSET_FILE_VERSION: u16 = 1;

        let mut typename: [u8; 4] = [0; 4];
        reader
            .read_exact(&mut typename)
            .map_err(|err| AssetRegistryError::ResourceIOError(primary_id, err))?;

        if &typename != ASSET_FILE_TYPENAME {
            return Err(AssetRegistryError::ResourceTypeMismatch(
                primary_id,
                format!("{:?}", typename),
                format!("{:?}", ASSET_FILE_TYPENAME),
            ));
        }

        // asset file header
        let version = reader
            .read_u16::<LittleEndian>()
            .map_err(|err| AssetRegistryError::ResourceIOError(primary_id, err))?;

        if version != ASSET_FILE_VERSION {
            return Err(AssetRegistryError::ResourceVersionMismatch(
                primary_id,
                version,
                ASSET_FILE_VERSION,
            ));
        }

        let reference_count = reader
            .read_u64::<LittleEndian>()
            .map_err(|err| AssetRegistryError::ResourceIOError(primary_id, err))?;

        let mut reference_list = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            let asset_ref = ResourceTypeAndId {
                kind: ResourceType::from_raw(
                    reader
                        .read_u64::<LittleEndian>()
                        .map_err(|err| AssetRegistryError::ResourceIOError(primary_id, err))?,
                ),
                id: ResourceId::from_raw(
                    reader
                        .read_u128::<LittleEndian>()
                        .map_err(|err| AssetRegistryError::ResourceIOError(primary_id, err))?,
                ),
            };
            reference_list.push(AssetReference {
                primary: asset_ref,
                secondary: asset_ref,
            });
        }

        // section header
        let asset_type = unsafe {
            std::mem::transmute::<u64, ResourceType>(
                reader.read_u64::<LittleEndian>().expect("valid data"),
            )
        };
        assert_eq!(
            asset_type, primary_id.kind,
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

        let loader = loaders
            .get_mut(&asset_type)
            .ok_or(AssetRegistryError::AssetLoaderNotFound(asset_type))?;

        let boxed_asset = loader
            .load(&mut &content[..])
            .map_err(|err| AssetRegistryError::AssetLoaderFailed(primary_id, err))?;

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
    use std::{sync::Arc, time::Duration};

    use lgn_content_store::{ContentProvider, ContentWriterExt, MemoryProvider};

    use super::{create_loader, AssetLoaderIO, AssetLoaderStub};
    use crate::{
        asset_loader::{HandleMap, LoaderRequest, LoaderResult},
        manifest::Manifest,
        test_asset, vfs, Handle, ResourceDescriptor, ResourceId, ResourceTypeAndId,
    };

    async fn setup_test() -> (ResourceTypeAndId, AssetLoaderStub, AssetLoaderIO) {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let (loader, mut io) = create_loader(vec![Box::new(vfs::CasDevice::new(
            Some(manifest),
            Arc::clone(&data_content_provider),
        ))]);
        io.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        (asset_id, loader, io)
    }

    #[tokio::test]
    async fn typed_ref() {
        let (asset_id, loader, mut io) = setup_test().await;

        {
            let untyped = loader.load(asset_id);

            assert_eq!(untyped.id(), asset_id);

            let typed: Handle<test_asset::TestAsset> = untyped.into();

            io.wait(Duration::from_millis(500)).await;
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
        io.wait(Duration::from_millis(500)).await;

        assert!(loader.handles.pin().get(&typed.id()).is_some());
    }

    #[tokio::test]
    async fn call_load_twice() {
        let (asset_id, loader, _io) = setup_test().await;

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

    #[tokio::test]
    async fn load_no_dependencies() {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let (unload_tx, _unload_rx) = crossbeam_channel::unbounded::<_>();

        let handles = HandleMap::new(unload_tx);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
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
        loader.wait(Duration::from_millis(1)).await;
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

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }

    #[tokio::test]
    async fn load_failed_dependency() {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let parent_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };

        let asset_id = {
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(parent_id, checksum);
            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
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
        loader.wait(Duration::from_millis(1)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::LoadError(_, _, _)));
    }

    #[tokio::test]
    async fn load_with_dependency() {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let parent_content = "parent";

        let parent_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };
        let child_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };

        let asset_id = {
            manifest.insert(
                child_id,
                data_content_provider
                    .write_content(&test_asset::tests::BINARY_ASSETFILE)
                    .await
                    .unwrap(),
            );
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(parent_id, checksum);

            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
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
        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

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

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

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

    #[tokio::test]
    async fn reload_no_dependencies() {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                Arc::clone(&data_content_provider),
            ))],
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
        loader.wait(Duration::from_millis(1)).await;
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
        loader.wait(Duration::from_millis(10)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Reloaded(_, _)));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }
}
