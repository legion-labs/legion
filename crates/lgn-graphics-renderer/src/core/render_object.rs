use crossbeam_channel::{Receiver, Sender};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use slab::Slab;
use std::{cmp::Ordering, sync::Arc};

// RenderObjectId

#[derive(Copy, Eq, PartialEq, Hash, Clone, Debug)]
pub struct RenderObjectId {
    feature_idx: u32,
    index: u32,
}

impl Ord for RenderObjectId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.feature_idx
            .cmp(&other.feature_idx)
            .then(self.index.cmp(&other.index))
    }
}

impl PartialOrd for RenderObjectId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for RenderObjectId {
    fn default() -> Self {
        Self {
            feature_idx: u32::MAX,
            index: u32::MAX,
        }
    }
}

// RenderObjectHandle

struct RenderObjectHandleInner {
    index: u32,
    sender: Sender<u32>,
}

impl Drop for RenderObjectHandleInner {
    fn drop(&mut self) {
        self.sender.send(self.index).unwrap();
    }
}

pub struct RenderObjectHandle {
    feature_idx: u32,
    inner: Arc<RenderObjectHandleInner>,
}

impl std::fmt::Debug for RenderObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderObjectHandle")
            .field("feature_idx", &self.feature_idx)
            .field("index", &self.inner.index)
            .finish()
    }
}

impl RenderObjectHandle {
    pub fn to_id(&self) -> RenderObjectId {
        RenderObjectId {
            index: self.inner.index,
            feature_idx: self.feature_idx,
        }
    }
}

// RenderObjectStorage

pub struct RenderObjectStorage<T> {
    sender: Sender<u32>,
    receiver: Receiver<u32>,
    objects: Slab<T>,
}

impl<RenderObjectStaticDataT> RenderObjectStorage<RenderObjectStaticDataT> {
    fn new(capacity: usize) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self {
            sender,
            receiver,
            objects: Slab::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    fn insert(&mut self, obj: RenderObjectStaticDataT) -> RenderObjectHandle {
        self.sync();
        let id = self.objects.insert(obj) as u32;
        RenderObjectHandle {
            feature_idx: 0,
            inner: Arc::new(RenderObjectHandleInner {
                index: id,
                sender: self.sender.clone(),
            }),
        }
    }

    pub fn get_from_id(&self, render_object_id: RenderObjectId) -> &RenderObjectStaticDataT {
        let key = render_object_id.index as usize;
        self.objects.get(key).unwrap_or_else(|| {
            panic!(
                "{} did not contain id {:?}.",
                std::any::type_name::<Self>(),
                key
            )
        })
    }

    pub fn get_from_handle(&self, handle: &RenderObjectHandle) -> &RenderObjectStaticDataT {
        let key = handle.inner.index as usize;
        self.objects.get(key).unwrap_or_else(|| {
            panic!(
                "{} did not contain handle {:?}.",
                std::any::type_name::<Self>(),
                handle
            )
        })
    }

    pub fn get_from_handle_mut(
        &mut self,
        handle: &RenderObjectHandle,
    ) -> &mut RenderObjectStaticDataT {
        let key = handle.inner.index as usize;
        self.objects.get_mut(key).unwrap_or_else(|| {
            panic!(
                "{} did not contain handle {:?}.",
                std::any::type_name::<Self>(),
                handle
            )
        })
    }

    fn sync(&mut self) {
        for index in self.receiver.try_iter() {
            self.objects.remove(index as usize);
        }
    }
}

// RenderObjectSet

pub struct RenderObjectSet<RenderObjectStaticDataT> {
    storage: Arc<RwLock<RenderObjectStorage<RenderObjectStaticDataT>>>,
}

impl<RenderObjectStaticDataT> RenderObjectSet<RenderObjectStaticDataT> {
    pub fn new(capacity: usize) -> Self {
        Self {
            storage: Arc::new(RwLock::new(RenderObjectStorage::new(capacity))),
        }
    }

    pub fn insert(&self, obj: RenderObjectStaticDataT) -> RenderObjectHandle {
        let handle = {
            let mut storage = self.write();
            storage.insert(obj)
        };
        handle
    }

    pub fn read(&self) -> RwLockReadGuard<'_, RenderObjectStorage<RenderObjectStaticDataT>> {
        self.storage.read()
    }

    fn write(&self) -> RwLockWriteGuard<'_, RenderObjectStorage<RenderObjectStaticDataT>> {
        self.storage.write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestData {
        value: u8,
    }

    #[test]
    fn test_basic_usage() {
        type MeshRenderObjectSet = RenderObjectSet<TestData>;

        let set = MeshRenderObjectSet::new(1);
        assert_eq!(set.read().len(), 0);

        let handle = set.insert(TestData { value: 13 });
        assert_eq!(set.read().len(), 1);

        // Test access by handle
        {
            assert_eq!(set.read().get_from_handle(&handle).value, 13);
        }

        // Test access by id
        {
            assert_eq!(set.read().get_from_id(handle.to_id()).value, 13);
        }

        drop(handle);

        set.write().sync();
        assert_eq!(set.read().len(), 0);
    }
}
