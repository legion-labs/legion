#![allow(missing_docs)] // todo

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc,
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
    assets: Vec<(AssetId, Option<Box<dyn Asset>>)>,
    load_dependencies: Vec<AssetReference>,
}

pub enum LoaderResult {
    Loaded(AssetId),
    Unloaded(AssetId),
    LoadError(AssetId, io::ErrorKind),
}

enum LoaderRequest {
    Load(AssetId, bool),
    Unload(AssetId, bool, Option<io::ErrorKind>),
}

struct LoaderPending {
    primary_id: AssetId,
    user_requested: bool,
    assets: Vec<(AssetId, Option<Box<dyn Asset>>)>,
    references: Vec<AssetReference>,
}

pub struct AssetLoader {
    creators: HashMap<AssetType, Box<dyn AssetCreator>>,

    request_pending: Vec<LoaderRequest>,
    request_await: Vec<LoaderPending>,

    /// Reference counts of primary and secondary assets.
    asset_refcounts: HashMap<AssetId, isize>,

    // this should be sent back to the game thread.
    asset_storage: HashMap<AssetId, Box<dyn Asset>>,

    /// List of secondary assets of a primary asset.
    secondary_assets: HashMap<AssetId, Vec<AssetId>>,

    /// List of primary asset's references to other primary assets .
    primary_asset_references: HashMap<AssetId, Vec<AssetId>>,

    /// Directory where the assets are located.
    ///
    /// todo: change to dyn CompiledAssetStore.
    work_dir: PathBuf,

    result_rx: mpsc::Sender<LoaderResult>,
}

// Asset loading:
// - add secondary asset information to `secondary_assets`
//     - for each secondary asset check if it is already loaded. always increase its reference count.
// - add primary asset references and schedule new loads.

impl AssetLoader {
    pub fn new(work_dir: impl AsRef<Path>, result_rx: mpsc::Sender<LoaderResult>) -> Self {
        Self {
            creators: HashMap::new(),
            request_pending: Vec::new(),
            request_await: Vec::new(),
            asset_refcounts: HashMap::new(),
            asset_storage: HashMap::new(),
            secondary_assets: HashMap::new(),
            primary_asset_references: HashMap::new(),
            work_dir: work_dir.as_ref().to_path_buf(),
            result_rx,
        }
    }
    pub fn register_creator(&mut self, kind: AssetType, creator: Box<dyn AssetCreator>) {
        self.creators.insert(kind, creator);
    }

    pub fn load_request(&mut self, id: AssetId) {
        self.request_pending.push(LoaderRequest::Load(id, true));
    }

    pub fn unload_request(&mut self, id: AssetId) -> bool {
        if let Some(r) = self.asset_refcounts.get_mut(&id) {
            *r -= 1;
            assert_eq!(*r, 0);
            self.request_pending
                .push(LoaderRequest::Unload(id, true, None));
            return true;
        }
        false
    }

    pub fn update(&mut self) -> usize {
        let mut internal_load_requests = vec![];

        // process new pending requests.
        let mut errors = vec![];
        for request in self.request_pending.drain(..) {
            match request {
                LoaderRequest::Load(primary_id, user_requested) => {
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
                                                *self.asset_refcounts.get_mut(asset_id).unwrap() +=
                                                    1;
                                            }
                                        }
                                    }
                                    internal_load_requests.extend(
                                        output.load_dependencies.iter().map(|reference| {
                                            LoaderRequest::Load(reference.primary, false)
                                        }),
                                    );
                                    self.request_await.push(LoaderPending {
                                        primary_id,
                                        user_requested,
                                        assets: output.assets,
                                        references: output.load_dependencies,
                                    });
                                }
                                Err(e) => errors.push((primary_id, user_requested, e)),
                            }
                        }
                        Err(e) => errors.push((primary_id, user_requested, e)),
                    }
                }
                LoaderRequest::Unload(primary_id, user_requested, err) => {
                    let r = self.asset_refcounts.remove(&primary_id).unwrap();
                    assert_eq!(r, 0);

                    if let Some(primary_references) =
                        self.primary_asset_references.remove(&primary_id)
                    {
                        if user_requested {
                            self.result_rx
                                .send(LoaderResult::Unloaded(primary_id))
                                .unwrap();
                        }

                        for ref_id in primary_references {
                            let r = self.asset_refcounts.get_mut(&ref_id).unwrap();
                            *r -= 1;
                            if *r == 0 {
                                // trigger internal unload
                                internal_load_requests
                                    .push(LoaderRequest::Unload(ref_id, false, err));
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
                }
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
                if failed_pending.user_requested {
                    self.result_rx
                        .send(LoaderResult::LoadError(
                            failed_pending.primary_id,
                            err.kind(),
                        ))
                        .unwrap();
                }
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
                        creator.load_init(boxed.as_mut());
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
                self.asset_storage
                    .extend(loaded.assets.into_iter().filter_map(|(id, asset)| {
                        if let Some(boxed) = asset {
                            return Some((id, boxed));
                        }
                        None
                    }));
                if loaded.user_requested {
                    // todo: assets are sent here but the 'finished' check above relies on assets being in self.assets
                    // do we send assets? do we send dependencies too?
                    self.result_rx
                        .send(LoaderResult::Loaded(loaded.primary_id))
                        .unwrap();
                }
            }
        }

        self.request_pending.extend(internal_load_requests);

        self.request_await.len() + self.request_pending.len()
    }

    fn load_internal(
        primary_id: AssetId,
        reader: &mut dyn io::Read,
        asset_refcounts: &HashMap<AssetId, isize>,
        creators: &mut HashMap<AssetType, Box<dyn AssetCreator>>,
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
            assets: vec![(primary_id, Some(boxed_asset))],
            load_dependencies: reference_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, sync::mpsc};

    use crate::{
        asset_loader::{asset_path, LoaderResult},
        test_asset::{self},
        AssetId,
    };

    use super::AssetLoader;

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

        let (rx, tx) = mpsc::channel::<LoaderResult>();

        let mut loader = AssetLoader::new(work_dir.path(), rx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        loader.load_request(asset_id);

        assert!(!loader.asset_storage.contains_key(&asset_id));

        assert!(loader.asset_refcounts.get(&asset_id).is_none());
        assert!(loader.secondary_assets.get(&asset_id).is_none());
        assert!(loader.primary_asset_references.get(&asset_id).is_none());

        let mut result = None;
        for _ in 0..5 {
            loader.update();
            if let Ok(res) = tx.try_recv() {
                result = Some(res);
                break;
            }
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_)));
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
        assert!(loader.unload_request(asset_id));
        for _ in 0..5 {
            loader.update(); // todo: wait for notification (make sure to reverse load order)
        }

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

        let (rx, tx) = mpsc::channel::<LoaderResult>();

        let mut loader = AssetLoader::new(work_dir.path(), rx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        loader.load_request(asset_id);

        assert!(!loader.asset_storage.contains_key(&asset_id));

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

        let mut result = None;
        for _ in 0..5 {
            loader.update();
            if let Ok(res) = tx.try_recv() {
                result = Some(res);
                break;
            }
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::LoadError(_, _)));
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

        let (rx, tx) = mpsc::channel::<LoaderResult>();

        let mut loader = AssetLoader::new(work_dir.path(), rx);
        loader.register_creator(
            test_asset::TYPE_ID,
            Box::new(test_asset::TestAssetCreator {}),
        );

        loader.load_request(asset_id);

        assert!(!loader.asset_storage.contains_key(&asset_id));

        assert!(loader.asset_refcounts.get(&parent_id).is_none());
        assert!(loader.secondary_assets.get(&parent_id).is_none());
        assert!(loader.primary_asset_references.get(&parent_id).is_none());

        let mut result = None;
        for _ in 0..5 {
            loader.update();
            if let Ok(res) = tx.try_recv() {
                result = Some(res);
                break;
            }
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_)));
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

        // unload and validate references.

        assert!(loader.unload_request(parent_id));
        for _ in 0..5 {
            loader.update(); // todo: wait for notification (make sure to reverse load order)
        }

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
