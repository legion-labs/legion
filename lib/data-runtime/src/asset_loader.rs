use std::{any::Any, collections::HashMap, io, time::Duration};

use crate::{vfs, AssetLoader, HandleId, HandleUntyped, RefOp, ResourceId, ResourceType};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: ResourceId,
    secondary: ResourceId,
}

/// The intermediate output of asset loading process.
///
/// Contains the result of loading a single file.
struct LoadOutput {
    /// None here means the asset was already loaded before so it doesn't have to be
    /// loaded again. It will still contribute to reference count though.
    assets: Vec<(ResourceId, Option<Box<dyn Any + Send + Sync>>)>,
    load_dependencies: Vec<AssetReference>,
}

pub(crate) enum LoaderResult {
    Loaded(ResourceId, Box<dyn Any + Send + Sync>, Option<LoadId>),
    Unloaded(ResourceId),
    LoadError(ResourceId, Option<LoadId>, io::ErrorKind),
}

pub(crate) enum LoaderRequest {
    Load(ResourceId, Option<LoadId>),
    Reload(ResourceId),
    Unload(ResourceId, bool, Option<io::ErrorKind>),
    Terminate,
}

/// State of a load request in progress.
struct LoadState {
    primary_id: ResourceId,
    /// If load_id is available it means the load was triggered by the user.
    /// Otherwise it is a load of a dependent Resource.
    load_id: Option<LoadId>,
    /// List of Resources in asset file identified by `primary_id`.
    /// None indicates a skipped secondary resource that was already loaded through another resource file.
    assets: Vec<(ResourceId, Option<Box<dyn Any + Send + Sync>>)>,
    /// The list of Resources that need to be loaded before the LoadState can be considered completed.
    references: Vec<AssetReference>,
}

pub(crate) fn create_loader(
    devices: Vec<Box<dyn vfs::Device>>,
) -> (AssetLoaderStub, AssetLoaderIO) {
    let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
    let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();

    let io = AssetLoaderIO::new(devices, request_tx.clone(), request_rx, result_tx);
    let loader = AssetLoaderStub::new(request_tx, result_rx);
    (loader, io)
}

pub(crate) struct AssetLoaderStub {
    id_generator: HandleId,
    refcount_channel: (
        crossbeam_channel::Sender<RefOp>,
        crossbeam_channel::Receiver<RefOp>,
    ),
    ref_counts: HashMap<HandleId, (ResourceId, isize)>,
    request_tx: crossbeam_channel::Sender<LoaderRequest>,
    result_rx: crossbeam_channel::Receiver<LoaderResult>,
}

type LoadId = u32;

impl AssetLoaderStub {
    fn new(
        request_tx: crossbeam_channel::Sender<LoaderRequest>,
        result_rx: crossbeam_channel::Receiver<LoaderResult>,
    ) -> Self {
        Self {
            id_generator: 0,
            refcount_channel: crossbeam_channel::unbounded(),
            ref_counts: HashMap::new(),
            request_tx,
            result_rx,
        }
    }

    pub(crate) fn get_handle(&mut self, id: ResourceId) -> Option<HandleUntyped> {
        self.ref_counts
            .iter_mut()
            .find_map(|(handle_id, (referred_id, count))| {
                if referred_id == &id {
                    *count += 1;
                    Some(*handle_id)
                } else {
                    None
                }
            })
            .map(|handle_id| HandleUntyped::create(handle_id, self.refcount_channel.0.clone()))
    }

    fn get_or_create_handle(&mut self, id: ResourceId) -> HandleUntyped {
        let handle_id = self
            .ref_counts
            .iter_mut()
            .find_map(|(handle_id, (referred_id, count))| {
                if referred_id == &id {
                    *count += 1;
                    Some(*handle_id)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                self.id_generator += 1;
                let new_id = self.id_generator;
                self.ref_counts.insert(new_id, (id, 1));
                new_id
            });
        HandleUntyped::create(handle_id, self.refcount_channel.0.clone())
    }

    pub(crate) fn process_refcount_ops(&mut self) -> Option<ResourceId> {
        while let Ok(op) = self.refcount_channel.1.try_recv() {
            match op {
                RefOp::AddRef(id) => {
                    let (_, count) = self.ref_counts.get_mut(&id).unwrap();
                    *count += 1;
                }
                RefOp::RemoveRef(id) => {
                    let (resource_id, count) = self.ref_counts.get_mut(&id).unwrap();
                    *count -= 1;
                    let resource_id = *resource_id;
                    if *count == 0 {
                        self.ref_counts.remove(&id);
                        return Some(resource_id);
                    }
                }
            }
        }
        None
    }

    /// Retrieves the asset id associated with a handle.
    pub(crate) fn get_asset_id(&self, handle_id: HandleId) -> Option<ResourceId> {
        self.ref_counts
            .get(&handle_id)
            .map(|(asset_id, _)| *asset_id)
    }

    pub(crate) fn terminate(&self) {
        self.request_tx.send(LoaderRequest::Terminate).unwrap();
    }

    pub(crate) fn load(&mut self, resource_id: ResourceId) -> HandleUntyped {
        let handle = self.get_or_create_handle(resource_id);
        self.request_tx
            .send(LoaderRequest::Load(resource_id, Some(handle.id)))
            .unwrap();
        handle
    }

    pub(crate) fn reload(&mut self, resource_id: ResourceId) -> bool {
        self.get_handle(resource_id).map_or(false, |_| {
            self.request_tx
                .send(LoaderRequest::Reload(resource_id))
                .unwrap();
            true
        })
    }

    pub(crate) fn try_result(&mut self) -> Option<LoaderResult> {
        self.result_rx.try_recv().ok()
    }

    pub(crate) fn unload(&mut self, id: ResourceId) {
        self.request_tx
            .send(LoaderRequest::Unload(id, true, None))
            .unwrap();
    }
}

const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";

pub(crate) struct AssetLoaderIO {
    loaders: HashMap<ResourceType, Box<dyn AssetLoader + Send>>,

    /// List of load requests waiting for all references to be loaded.
    processing_list: Vec<LoadState>,

    /// Reference counts of primary and secondary assets.
    asset_refcounts: HashMap<ResourceId, isize>,

    /// List of secondary assets of a primary asset.
    secondary_assets: HashMap<ResourceId, Vec<ResourceId>>,

    /// List of primary asset's references to other primary assets .
    primary_asset_references: HashMap<ResourceId, Vec<ResourceId>>,

    devices: Vec<Box<dyn vfs::Device>>,

    /// Loopback for load requests.
    request_tx: crossbeam_channel::Sender<LoaderRequest>,

    /// Entry point for load requests.
    request_rx: Option<crossbeam_channel::Receiver<LoaderRequest>>,

    /// Output of loader results.
    result_tx: crossbeam_channel::Sender<LoaderResult>,
}

// Asset loading:
// - add secondary asset information to `secondary_assets`
//     - for each secondary asset check if it is already loaded. always increase its reference count.
// - add primary asset references and schedule new loads.

impl AssetLoaderIO {
    pub(crate) fn new(
        devices: Vec<Box<dyn vfs::Device>>,
        request_tx: crossbeam_channel::Sender<LoaderRequest>,
        request_rx: crossbeam_channel::Receiver<LoaderRequest>,
        result_tx: crossbeam_channel::Sender<LoaderResult>,
    ) -> Self {
        Self {
            loaders: HashMap::new(),
            processing_list: Vec::new(),
            asset_refcounts: HashMap::new(),
            devices,
            secondary_assets: HashMap::new(),
            primary_asset_references: HashMap::new(),
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

    fn read_resource(&self, id: ResourceId) -> io::Result<Vec<u8>> {
        for device in &self.devices {
            if let Some(content) = device.lookup(id) {
                return Ok(content);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Resource Not Found",
        ))
    }

    fn process_reload(&mut self, primary_id: ResourceId) -> Result<(), io::Error> {
        let asset_data = self.read_resource(primary_id)?;

        // for now it should be empty. later - cause a reload of secondary assets?
        //
        let _secondary_assets = self.secondary_assets.get(&primary_id).unwrap();

        // compare against the header. decrease relevant refcount -> cause unload? cause a new load? ;/
        let _primary_asset_references = self.primary_asset_references.get(&primary_id).unwrap();

        // todo: call load_init after a reload.

        let load_func = {
            if asset_data.len() < 4 || &asset_data[0..4] != ASSET_FILE_TYPENAME {
                Self::load_raw
            } else {
                Self::load_asset_file
            }
        };

        let output = load_func(primary_id, &mut &asset_data[..], &mut self.loaders)?;

        assert_eq!(
            output.assets.len(),
            1,
            "Reload of secondary assets not supported"
        );

        for (asset_id, asset) in &output.assets {
            match asset {
                Some(_) => {
                    let res = self.asset_refcounts.insert(*asset_id, 1);
                    assert!(res.is_none(), "Asset was already loaded.");
                }
                None => {
                    *self.asset_refcounts.get_mut(asset_id).unwrap() += 1;
                }
            }
        }
        Ok(())
    }

    fn process_load(
        &mut self,
        primary_id: ResourceId,
        load_id: Option<u32>,
    ) -> Result<(), (ResourceId, Option<LoadId>, io::Error)> {
        if self.asset_refcounts.contains_key(&primary_id) {
            // todo: we should create a LoadState based on existing load state?
            // this way the load result will be notified when the resource is actually loaded.
            return Ok(());
        }
        let asset_data = self
            .read_resource(primary_id)
            .map_err(|e| (primary_id, load_id, e))?;

        let load_func = {
            if asset_data.len() < 4 || &asset_data[0..4] != ASSET_FILE_TYPENAME {
                Self::load_raw
            } else {
                Self::load_asset_file
            }
        };

        let output = load_func(primary_id, &mut &asset_data[..], &mut self.loaders)
            .map_err(|e| (primary_id, load_id, e))?;

        for (asset_id, asset) in &output.assets {
            match asset {
                Some(_) => {
                    let res = self.asset_refcounts.insert(*asset_id, 1);
                    assert!(res.is_none());
                }
                None => {
                    *self.asset_refcounts.get_mut(asset_id).unwrap() += 1;
                }
            }
        }
        for reference in &output.load_dependencies {
            self.request_tx
                .send(LoaderRequest::Load(reference.primary, None))
                .unwrap();
        }
        self.processing_list.push(LoadState {
            primary_id,
            load_id,
            assets: output.assets,
            references: output.load_dependencies,
        });
        Ok(())
    }

    fn process_unload(
        &mut self,
        primary_id: ResourceId,
        user_requested: bool,
        err: Option<std::io::ErrorKind>,
    ) {
        if let Some(r) = self.asset_refcounts.remove(&primary_id) {
            assert!(r <= 1);

            if let Some(primary_references) = self.primary_asset_references.remove(&primary_id) {
                if user_requested {
                    self.result_tx
                        .send(LoaderResult::Unloaded(primary_id))
                        .unwrap();
                }

                for ref_id in primary_references {
                    let r = self.asset_refcounts.get_mut(&ref_id).unwrap();
                    *r -= 1;
                    if *r == 0 {
                        // trigger internal unload
                        self.request_tx
                            .send(LoaderRequest::Unload(ref_id, false, err))
                            .unwrap();
                    }
                }
            }
            if let Some(secondary_assets) = self.secondary_assets.remove(&primary_id) {
                for id in secondary_assets {
                    let r = self.asset_refcounts.get_mut(&id).unwrap();
                    *r -= 1;
                    if *r == 0 {
                        self.asset_refcounts.remove(&id);
                        // todo: tell the user.
                    }
                }
            }
        } else {
            // todo(kstatik): tell the user that the id is invalid
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_request(
        &mut self,
        request: LoaderRequest,
    ) -> Result<(), (ResourceId, Option<LoadId>, io::Error)> {
        match request {
            LoaderRequest::Load(primary_id, load_id) => self.process_load(primary_id, load_id),
            LoaderRequest::Reload(primary_id) => self
                .process_reload(primary_id)
                .map_err(|e| (primary_id, None, e)),
            LoaderRequest::Unload(primary_id, user_requested, err) => {
                self.process_unload(primary_id, user_requested, err);
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
        for (failed_asset_id, _, err) in errors {
            let (failed, pending): (Vec<_>, Vec<_>) = std::mem::take(&mut self.processing_list)
                .into_iter()
                .partition(|pending| {
                    pending
                        .references
                        .iter()
                        .any(|r| r.primary == failed_asset_id)
                });

            for failed_pending in failed {
                self.result_tx
                    .send(LoaderResult::LoadError(
                        failed_pending.primary_id,
                        failed_pending.load_id,
                        err.kind(),
                    ))
                    .unwrap();
            }
            self.result_tx
                .send(LoaderResult::LoadError(failed_asset_id, None, err.kind()))
                .unwrap();

            self.processing_list = pending;
        }

        // check for completion.
        for index in (0..self.processing_list.len()).rev() {
            let pending = &self.processing_list[index];
            let finished = pending
                .references
                .iter()
                .all(|reference| self.asset_refcounts.contains_key(&reference.primary));
            if finished {
                let mut loaded = self.processing_list.swap_remove(index);

                for (asset_id, asset) in &mut loaded.assets {
                    if let Some(boxed) = asset {
                        let loader = self.loaders.get_mut(&asset_id.ty()).unwrap();
                        loader.load_init(boxed.as_mut());
                    }
                    // if there is no boxed asset here, it means it was already loaded before.
                }

                self.primary_asset_references.insert(
                    loaded.primary_id,
                    loaded.references.iter().map(|e| e.primary).collect(),
                );

                self.secondary_assets.insert(
                    loaded.primary_id,
                    loaded
                        .assets
                        .iter()
                        .skip(1)
                        .map(|(id, _)| id)
                        .copied()
                        .collect(),
                );
                // send primary asset with load_id. all secondary assets without to not cause load notification.
                let mut asset_iter = loaded.assets.into_iter();
                let primary_asset = asset_iter.next().unwrap().1.unwrap();
                self.result_tx
                    .send(LoaderResult::Loaded(
                        loaded.primary_id,
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
        id: ResourceId,
        reader: &mut dyn io::Read,
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send>>,
    ) -> Result<LoadOutput, io::Error> {
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;

        let asset_type = id.ty();
        let loader = loaders.get_mut(&asset_type).unwrap();
        let boxed_asset = loader.load(&mut &content[..])?;

        Ok(LoadOutput {
            assets: vec![(id, Some(boxed_asset))],
            load_dependencies: vec![],
        })
    }

    fn load_asset_file(
        primary_id: ResourceId,
        reader: &mut dyn io::Read,
        loaders: &mut HashMap<ResourceType, Box<dyn AssetLoader + Send>>,
    ) -> Result<LoadOutput, io::Error> {
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
            let asset_ref = unsafe {
                std::mem::transmute::<u128, ResourceId>(reader.read_u128::<LittleEndian>()?)
            };
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
            asset_type,
            primary_id.ty(),
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
    use std::{thread, time::Duration};

    use legion_content_store::{ContentStore, RamContentStore};

    use crate::{
        asset_loader::{LoaderRequest, LoaderResult},
        manifest::Manifest,
        test_asset, vfs, Handle, Resource, ResourceId,
    };

    use super::{create_loader, AssetLoaderIO, AssetLoaderStub};

    fn setup_test() -> (ResourceId, AssetLoaderStub, AssetLoaderIO) {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_assetfile = [
            97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
        ];

        let asset_id = {
            let id = ResourceId::new(test_asset::TestAsset::TYPE, 1);
            let checksum = content_store.store(&binary_assetfile).unwrap();
            manifest.insert(id, checksum, binary_assetfile.len());
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
    fn ref_count() {
        let (asset_id, mut loader, _io) = setup_test();

        let internal_id;
        {
            let a = loader.load(asset_id);
            internal_id = a.id;
            assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 1);

            {
                let b = a.clone();
                while loader.process_refcount_ops().is_some() {}

                assert_eq!(loader.ref_counts.get(&b.id).unwrap().1, 2);
                assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 2);
                assert_eq!(a, b);
            }
            while loader.process_refcount_ops().is_some() {}
            assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 1);
        }
        while loader.process_refcount_ops().is_some() {}
        assert!(!loader.ref_counts.contains_key(&internal_id));
    }

    #[test]
    fn typed_ref() {
        let (asset_id, mut loader, _io) = setup_test();

        let internal_id;
        {
            let untyped = loader.load(asset_id);
            assert_eq!(loader.ref_counts.get(&untyped.id).unwrap().1, 1);

            internal_id = untyped.id;

            let typed: Handle<test_asset::TestAsset> = untyped.into();
            while loader.process_refcount_ops().is_some() {}
            assert_eq!(loader.ref_counts.get(&typed.id).unwrap().1, 1);

            let mut test_timeout = Duration::from_millis(500);
            while test_timeout > Duration::ZERO && loader.ref_counts.get(&typed.id).is_none() {
                let sleep_time = Duration::from_millis(10);
                thread::sleep(sleep_time);
                test_timeout -= sleep_time;
                while loader.process_refcount_ops().is_some() {}
            }
            assert!(loader.ref_counts.get(&typed.id).is_some());
        }

        while loader.process_refcount_ops().is_some() {} // to drop the refcount to zero.

        assert!(!loader.ref_counts.contains_key(&internal_id));

        let typed: Handle<test_asset::TestAsset> = loader.load(asset_id).into();

        let mut test_timeout = Duration::from_millis(500);
        while test_timeout > Duration::ZERO && loader.ref_counts.get(&typed.id).is_none() {
            let sleep_time = Duration::from_millis(10);
            thread::sleep(sleep_time);
            test_timeout -= sleep_time;
            while loader.process_refcount_ops().is_some() {}
        }
        assert!(loader.ref_counts.get(&typed.id).is_some());
    }

    #[test]
    fn call_load_twice() {
        let (asset_id, mut loader, _io) = setup_test();

        let a = loader.load(asset_id);
        assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 1);
        {
            let b = a.clone();
            while loader.process_refcount_ops().is_some() {}
            assert_eq!(a.id, b.id);
            assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 2);
            {
                let c = loader.load(asset_id);
                assert_eq!(a.id, c.id);
                assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 3);
            }
            while loader.process_refcount_ops().is_some() {}
            assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 2);
        }
        while loader.process_refcount_ops().is_some() {}
        assert_eq!(loader.ref_counts.get(&a.id).unwrap().1, 1);
    }

    #[test]
    fn load_no_dependencies() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_assetfile = [
            97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
        ];

        let asset_id = {
            let id = ResourceId::new(test_asset::TestAsset::TYPE, 1);
            let checksum = content_store.store(&binary_assetfile).unwrap();
            manifest.insert(id, checksum, binary_assetfile.len());
            id
        };

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("to send request");

        assert!(loader.asset_refcounts.get(&asset_id).is_none());
        assert!(loader.secondary_assets.get(&asset_id).is_none());
        assert!(loader.primary_asset_references.get(&asset_id).is_none());

        let mut result = None;
        loader.wait(Duration::from_millis(1));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert_eq!(loader.asset_refcounts.get(&asset_id).unwrap(), &1);
        assert_eq!(loader.secondary_assets.get(&asset_id).unwrap().len(), 0);
        assert_eq!(
            loader
                .primary_asset_references
                .get(&asset_id)
                .unwrap()
                .len(),
            0
        );

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id, true, None))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(loader.asset_refcounts.get(&asset_id).is_none());
        assert!(loader.secondary_assets.get(&asset_id).is_none());
        assert!(loader.primary_asset_references.get(&asset_id).is_none());
    }

    #[test]
    fn load_failed_dependency() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_parent_assetfile = [
            97, 115, 102, 116, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112,
            97, 114, 101, 110, 116,
        ];

        let parent_id = ResourceId::new(test_asset::TestAsset::TYPE, 2);

        let asset_id = {
            let checksum = content_store.store(&binary_parent_assetfile).unwrap();
            manifest.insert(parent_id, checksum, binary_parent_assetfile.len());
            parent_id
        };

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("valid tx");

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

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

        let binary_parent_assetfile = [
            97, 115, 102, 116, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112,
            97, 114, 101, 110, 116,
        ];
        let binary_child_assetfile = [
            97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
        ];
        let parent_content = "parent";

        let parent_id = ResourceId::new(test_asset::TestAsset::TYPE, 2);
        let child_id = ResourceId::new(test_asset::TestAsset::TYPE, 1);

        let asset_id = {
            manifest.insert(
                child_id,
                content_store.store(&binary_child_assetfile).unwrap(),
                binary_child_assetfile.len(),
            );
            let checksum = content_store.store(&binary_parent_assetfile).unwrap();
            manifest.insert(parent_id, checksum, binary_parent_assetfile.len());

            parent_id
        };

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("to send request");

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

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
        assert_eq!(loader.asset_refcounts.get(&parent_id).unwrap(), &1);
        assert_eq!(loader.asset_refcounts.get(&child_id).unwrap(), &1);
        assert_eq!(loader.secondary_assets.get(&parent_id).unwrap().len(), 0);
        assert_eq!(
            loader
                .primary_asset_references
                .get(&parent_id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            loader.primary_asset_references.get(&parent_id).unwrap()[0],
            child_id
        );
        assert_eq!(loader.secondary_assets.get(&child_id).unwrap().len(), 0);
        assert_eq!(
            loader
                .primary_asset_references
                .get(&child_id)
                .unwrap()
                .len(),
            0
        );

        if let LoaderResult::Loaded(id, asset, returned_load_id) = result {
            let asset = asset.downcast_ref::<test_asset::TestAsset>().unwrap();
            assert_eq!(asset.content, parent_content);
            assert_eq!(asset_id, id);
            assert_eq!(returned_load_id, load_id);
        }

        // unload and validate references.

        request_tx
            .send(LoaderRequest::Unload(parent_id, true, None))
            .expect("to send request");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

        /*
            assert_eq!(result.assets.len(), 1);
            assert_eq!(result._load_dependencies.len(), 1);

            let (asset_id, asset) = &result.assets[0];

            let asset = asset.downcast_ref::<TestAsset>().unwrap();
            assert_eq!(asset.content, expected_content);
            assert_eq!(asset_id, &id);
        */
    }

    //#[test]
    #[allow(dead_code)]
    fn reload_no_dependencies() {
        let mut content_store = Box::new(RamContentStore::default());
        let mut manifest = Manifest::default();

        let binary_assetfile = [
            97, 115, 102, 116, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 0, 0, 0, 99, 104, 105, 108, 100,
        ];

        let asset_id = {
            let id = ResourceId::new(test_asset::TestAsset::TYPE, 1);
            let checksum = content_store.store(&binary_assetfile).unwrap();
            manifest.insert(id, checksum, binary_assetfile.len());
            id
        };

        let (request_tx, request_rx) = crossbeam_channel::unbounded::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(manifest, content_store))],
            request_tx.clone(),
            request_rx,
            result_tx,
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);

        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("to send request");

        assert!(loader.asset_refcounts.get(&asset_id).is_none());
        assert!(loader.secondary_assets.get(&asset_id).is_none());
        assert!(loader.primary_asset_references.get(&asset_id).is_none());

        let mut result = None;
        loader.wait(Duration::from_millis(1));
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert_eq!(loader.asset_refcounts.get(&asset_id).unwrap(), &1);
        assert_eq!(loader.secondary_assets.get(&asset_id).unwrap().len(), 0);
        assert_eq!(
            loader
                .primary_asset_references
                .get(&asset_id)
                .unwrap()
                .len(),
            0
        );

        // reload
        request_tx.send(LoaderRequest::Reload(asset_id)).unwrap();

        loader.wait(Duration::from_millis(10));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id, true, None))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}

        assert!(loader.asset_refcounts.get(&asset_id).is_none());
        assert!(loader.secondary_assets.get(&asset_id).is_none());
        assert!(loader.primary_asset_references.get(&asset_id).is_none());
    }
}
