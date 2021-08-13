use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    time::Duration,
};

use crate::{Asset, AssetCreator, AssetId, AssetType};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

fn asset_path(id: AssetId) -> PathBuf {
    PathBuf::from(id.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetReference {
    primary: AssetId,
    secondary: AssetId,
}

/// The intermediate output of asset loading process.
///
/// Contains the result of loading a single file.
struct LoadOutput {
    assets: Vec<(AssetId, Option<Arc<dyn Asset + Send + Sync>>)>,
    load_dependencies: Vec<AssetReference>,
}

pub(crate) enum LoaderResult {
    Loaded(AssetId, Arc<dyn Asset + Send + Sync>, Option<LoadId>),
    Unloaded(AssetId),
    LoadError(AssetId, Option<LoadId>, io::ErrorKind),
}

pub(crate) enum LoaderRequest {
    Load(AssetId, Option<LoadId>),
    Unload(AssetId, bool, Option<io::ErrorKind>),
    Terminate,
}

struct LoaderPending {
    primary_id: AssetId,
    load_id: Option<LoadId>,
    assets: Vec<(AssetId, Option<Arc<dyn Asset + Send + Sync>>)>,
    references: Vec<AssetReference>,
}

pub(crate) fn create_loader(work_dir: PathBuf) -> (AssetLoader, AssetLoaderIO) {
    let (result_tx, result_rx) = mpsc::channel::<LoaderResult>();
    let (request_tx, request_rx) = mpsc::channel::<LoaderRequest>();

    let io = AssetLoaderIO::new(work_dir, request_tx.clone(), request_rx, result_tx);
    let loader = AssetLoader::new(request_tx, result_rx);
    (loader, io)
}

pub(crate) struct AssetLoader {
    request_tx: mpsc::Sender<LoaderRequest>,
    result_rx: mpsc::Receiver<LoaderResult>,
}

type LoadId = u32;

impl AssetLoader {
    fn new(
        request_tx: mpsc::Sender<LoaderRequest>,
        result_rx: mpsc::Receiver<LoaderResult>,
    ) -> Self {
        Self {
            request_tx,
            result_rx,
        }
    }

    pub(crate) fn terminate(&self) {
        self.request_tx.send(LoaderRequest::Terminate).unwrap();
    }

    pub(crate) fn load(&self, asset_id: AssetId, load_id: LoadId) {
        // todo: pass HandleId
        self.request_tx
            .send(LoaderRequest::Load(asset_id, Some(load_id)))
            .unwrap();
    }

    pub(crate) fn try_result(&mut self) -> Option<LoaderResult> {
        self.result_rx.try_recv().ok()
    }
    pub(crate) fn unload(&mut self, id: AssetId) {
        self.request_tx
            .send(LoaderRequest::Unload(id, true, None))
            .unwrap();
    }
}

pub(crate) struct AssetLoaderIO {
    creators: HashMap<AssetType, Box<dyn AssetCreator + Send>>,

    request_await: Vec<LoaderPending>,

    /// Reference counts of primary and secondary assets.
    asset_refcounts: HashMap<AssetId, isize>,

    // this should be sent back to the game thread.
    asset_storage: HashMap<AssetId, Arc<dyn Asset + Send + Sync>>,

    /// List of secondary assets of a primary asset.
    secondary_assets: HashMap<AssetId, Vec<AssetId>>,

    /// List of primary asset's references to other primary assets .
    primary_asset_references: HashMap<AssetId, Vec<AssetId>>,

    /// Directory where the assets are located.
    ///
    /// todo: change to dyn CompiledAssetStore.
    work_dir: PathBuf,

    request_tx: mpsc::Sender<LoaderRequest>,
    request_rx: Option<mpsc::Receiver<LoaderRequest>>,
    result_tx: mpsc::Sender<LoaderResult>,
}

// Asset loading:
// - add secondary asset information to `secondary_assets`
//     - for each secondary asset check if it is already loaded. always increase its reference count.
// - add primary asset references and schedule new loads.

impl AssetLoaderIO {
    pub(crate) fn new(
        work_dir: impl AsRef<Path>,
        request_tx: mpsc::Sender<LoaderRequest>,
        request_rx: mpsc::Receiver<LoaderRequest>,
        result_tx: mpsc::Sender<LoaderResult>,
    ) -> Self {
        Self {
            creators: HashMap::new(),
            request_await: Vec::new(),
            asset_refcounts: HashMap::new(),
            asset_storage: HashMap::new(),
            secondary_assets: HashMap::new(),
            primary_asset_references: HashMap::new(),
            work_dir: work_dir.as_ref().to_path_buf(),
            request_tx,
            request_rx: Some(request_rx),
            result_tx,
        }
    }
    pub(crate) fn register_creator(
        &mut self,
        kind: AssetType,
        creator: Box<dyn AssetCreator + Send>,
    ) {
        self.creators.insert(kind, creator);
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process(&mut self, request: LoaderRequest) -> Option<(AssetId, Option<LoadId>, io::Error)> {
        match request {
            LoaderRequest::Load(primary_id, load_id) => {
                let file_path = self.work_dir.join(asset_path(primary_id));
                match fs::File::open(file_path) {
                    Ok(mut file) => {
                        match Self::load_internal(
                            primary_id,
                            &mut file,
                            &self.asset_refcounts,
                            &mut self.creators,
                        ) {
                            Ok(output) => {
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
                                self.request_await.push(LoaderPending {
                                    primary_id,
                                    load_id,
                                    assets: output.assets,
                                    references: output.load_dependencies,
                                });
                                None
                            }
                            Err(e) => Some((primary_id, load_id, e)),
                        }
                    }
                    Err(e) => Some((primary_id, load_id, e)),
                }
            }
            LoaderRequest::Unload(primary_id, user_requested, err) => {
                let r = self.asset_refcounts.remove(&primary_id).unwrap();
                assert!(r <= 1);

                if let Some(primary_references) = self.primary_asset_references.remove(&primary_id)
                {
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
                None
            }
            LoaderRequest::Terminate => {
                self.request_rx = None;
                None
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
                    Err(mpsc::RecvTimeoutError::Disconnected) => return None,
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                    Ok(request) => {
                        if let Some(error) = self.process(request) {
                            errors.push(error);
                        }
                    }
                },
            }
        }

        // todo: propagate errors to dependent assets before sending results.
        for (failed_asset_id, _, err) in errors {
            let (failed, pending): (Vec<_>, Vec<_>) = std::mem::take(&mut self.request_await)
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
            self.request_await = pending;
        }

        // check for completion.
        for index in (0..self.request_await.len()).rev() {
            let pending = &self.request_await[index];
            let finished = pending
                .references
                .iter()
                .all(|reference| self.asset_storage.contains_key(&reference.primary));
            if finished {
                let mut loaded = self.request_await.swap_remove(index);

                for (asset_id, asset) in &mut loaded.assets {
                    if let Some(boxed) = asset {
                        let creator = self.creators.get_mut(&asset_id.asset_type()).unwrap();

                        // SAFETY: this is safe because loaded asset is only referenced by the loader.
                        // it hasn't been made available to other systems yet.
                        //let boxed = unsafe { Arc::get_mut_unchecked(boxed) };
                        let boxed = Arc::get_mut(boxed).unwrap();
                        creator.load_init(boxed);
                    }
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
                        .cloned()
                        .collect(),
                );
                let primary_asset = loaded.assets[0].1.as_ref().unwrap().clone();
                self.asset_storage
                    .extend(loaded.assets.into_iter().filter_map(|(id, asset)| {
                        if let Some(boxed) = asset {
                            return Some((id, boxed));
                        }
                        None
                    }));
                if loaded.load_id.is_some() {
                    self.result_tx
                        .send(LoaderResult::Loaded(
                            loaded.primary_id,
                            primary_asset,
                            loaded.load_id,
                        ))
                        .unwrap();
                }
            }
        }

        Some(self.request_await.len())
    }

    fn load_internal(
        primary_id: AssetId,
        reader: &mut dyn io::Read,
        asset_refcounts: &HashMap<AssetId, isize>,
        creators: &mut HashMap<AssetType, Box<dyn AssetCreator + Send>>,
    ) -> Result<LoadOutput, io::Error> {
        const ASSET_FILE_VERSION: u16 = 1;

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
            let asset_ref =
                unsafe { std::mem::transmute::<u64, AssetId>(reader.read_u64::<LittleEndian>()?) };
            reference_list.push(AssetReference {
                primary: asset_ref,
                secondary: asset_ref,
            });
        }

        // todo: if asset is already loaded it should be skipped as pass as 'None'
        assert!(!asset_refcounts.contains_key(&primary_id));

        // section header
        let asset_type = unsafe {
            std::mem::transmute::<u32, AssetType>(
                reader.read_u32::<LittleEndian>().expect("valid data"),
            )
        };
        let asset_count = reader.read_u64::<LittleEndian>().expect("valid data");
        assert_eq!(asset_count, 1);

        let nbytes = reader.read_u64::<LittleEndian>().expect("valid data");

        let mut content = Vec::new();
        content.resize(nbytes as usize, 0);
        reader.read_exact(&mut content).expect("valid data");

        let creator = creators.get_mut(&asset_type).unwrap();
        let boxed_asset = creator.load(asset_type, &mut &content[..]).unwrap();

        Ok(LoadOutput {
            assets: vec![(primary_id, Some(Arc::from(boxed_asset)))],
            load_dependencies: reference_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, sync::mpsc, time::Duration};

    use crate::{
        asset_loader::{asset_path, LoaderRequest, LoaderResult},
        test_asset::{self},
        AssetId,
    };

    use super::AssetLoaderIO;

    #[test]
    fn load_no_dependencies() {
        let work_dir = tempfile::tempdir().unwrap();

        let binary_assetfile = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0,
            0, 0, 0, 99, 104, 105, 108, 100,
        ];

        let asset_id = {
            let asset_id = AssetId::new(test_asset::TYPE_ID, 1);
            let asset_path = work_dir.path().join(asset_path(asset_id));
            let mut file = fs::File::create(asset_path).expect("new file");
            file.write_all(&binary_assetfile).expect("successful write");
            asset_id
        };

        let (request_tx, request_rx) = mpsc::channel::<LoaderRequest>();
        let (result_tx, result_rx) = mpsc::channel::<LoaderResult>();
        let mut loader =
            AssetLoaderIO::new(work_dir.path(), request_tx.clone(), request_rx, result_tx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("to send request");

        assert!(!loader.asset_storage.contains_key(&asset_id));

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
        assert!(loader.asset_storage.contains_key(&asset_id));
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
        let work_dir = tempfile::tempdir().unwrap();

        let binary_parent_assetfile = [
            1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0,
            0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112, 97, 114, 101, 110, 116,
        ];

        let parent_id = AssetId::new(test_asset::TYPE_ID, 2);

        let asset_id = {
            let parent_path = work_dir.path().join(asset_path(parent_id));

            let mut file = fs::File::create(parent_path).expect("new file");
            file.write_all(&binary_parent_assetfile)
                .expect("successful write");
            parent_id
        };

        let (request_tx, request_rx) = mpsc::channel::<LoaderRequest>();
        let (result_tx, result_rx) = mpsc::channel::<LoaderResult>();
        let mut loader =
            AssetLoaderIO::new(work_dir.path(), request_tx.clone(), request_rx, result_tx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("valid tx");

        assert!(!loader.asset_storage.contains_key(&asset_id));

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
        let work_dir = tempfile::tempdir().unwrap();

        let binary_parent_assetfile = [
            1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 86, 63, 214, 53, 86, 63, 214, 53, 1, 0, 0, 0,
            0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 112, 97, 114, 101, 110, 116,
        ];
        let binary_child_assetfile = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 63, 214, 53, 1, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0,
            0, 0, 0, 99, 104, 105, 108, 100,
        ];
        let parent_content = "parent";

        let parent_id = AssetId::new(test_asset::TYPE_ID, 2);
        let child_id = AssetId::new(test_asset::TYPE_ID, 1);

        let asset_id = {
            let parent_path = work_dir.path().join(asset_path(parent_id));

            let mut file = fs::File::create(parent_path).expect("new file");
            file.write_all(&binary_parent_assetfile)
                .expect("successful write");

            let child_path = work_dir.path().join(asset_path(child_id));
            let mut file = fs::File::create(child_path).expect("new file");
            file.write_all(&binary_child_assetfile)
                .expect("successful write");

            parent_id
        };

        let (request_tx, request_rx) = mpsc::channel::<LoaderRequest>();
        let (result_tx, result_rx) = mpsc::channel::<LoaderResult>();
        let mut loader =
            AssetLoaderIO::new(work_dir.path(), request_tx.clone(), request_rx, result_tx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_id, load_id))
            .expect("to send request");

        assert!(!loader.asset_storage.contains_key(&asset_id));

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

        let mut result = None;
        while loader.wait(Duration::from_millis(1)).unwrap() > 0 {}
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(matches!(result, LoaderResult::Loaded(_, _, _)));
        assert!(loader.asset_storage.contains_key(&asset_id));
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
            let asset = asset
                .as_any()
                .downcast_ref::<test_asset::TestAsset>()
                .unwrap();
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

            let asset = asset.as_any().downcast_ref::<TestAsset>().unwrap();
            assert_eq!(asset.content, expected_content);
            assert_eq!(asset_id, &id);
        */
    }
}
