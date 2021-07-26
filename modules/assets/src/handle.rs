use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{Asset, AssetRegistry};

/// Internal-only runtime id of the asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AssetHandleId(pub u64);

/// A thread-safe reference-counting handle to the asset.
///
/// Only when the reference count reaches zero the `AssetManager` will be able to release the asset.
#[derive(Debug)]
pub struct AssetGenericHandle {
    shared: Arc<Mutex<AssetRefCounter>>,
    pub(crate) handle_id: AssetHandleId,
}

impl PartialEq for AssetGenericHandle {
    fn eq(&self, other: &Self) -> bool {
        self.handle_id == other.handle_id
    }
}

impl Eq for AssetGenericHandle {}

impl AssetGenericHandle {
    pub(crate) fn new(counter: Arc<Mutex<AssetRefCounter>>, id: AssetHandleId) -> Self {
        counter.lock().unwrap().increase(id);
        Self {
            shared: counter,
            handle_id: id,
        }
    }

    /// Returns a reference to a usable asset.
    ///
    /// The method will return None if the asset is not yet loaded or processed.
    pub fn get<'a, A: Asset>(&'_ self, assets: &'a AssetRegistry) -> Option<&'a A> {
        assets.get(self)
    }
}

impl Drop for AssetGenericHandle {
    fn drop(&mut self) {
        self.shared.lock().unwrap().decrease(self.handle_id);
    }
}

impl Clone for AssetGenericHandle {
    fn clone(&self) -> Self {
        self.shared.lock().unwrap().increase(self.handle_id);
        Self {
            shared: self.shared.clone(),
            handle_id: self.handle_id,
        }
    }
}

/// Data shared between all the handles and the `AssetRegistry`.
#[derive(Debug)]
pub(crate) struct AssetRefCounter {
    refs: HashMap<AssetHandleId, isize>,
    orphans: Vec<AssetHandleId>,
}

impl AssetRefCounter {
    pub(crate) fn new() -> Self {
        Self {
            refs: HashMap::new(),
            orphans: Vec::new(),
        }
    }

    fn increase(&mut self, handle_id: AssetHandleId) {
        if let Some(value) = self.refs.get_mut(&handle_id) {
            *value += 1;
        } else {
            self.orphans.retain(|asset_id| asset_id != &handle_id);
            self.refs.insert(handle_id, 1);
        }
    }

    fn decrease(&mut self, handle_id: AssetHandleId) {
        let count = self.refs.get_mut(&handle_id).unwrap();
        *count -= 1;
        if *count == 0 {
            self.refs.remove(&handle_id);
            self.orphans.push(handle_id);
        }
    }

    pub(crate) fn orphans(&mut self) -> Vec<AssetHandleId> {
        std::mem::take(&mut self.orphans)
    }
}
