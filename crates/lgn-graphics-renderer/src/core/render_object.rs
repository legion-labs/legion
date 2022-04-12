use crossbeam_channel::*;
use slab::*;
use std::marker::PhantomData;

pub struct RenderObjectHandle<T> {
    id: u32,
    sender: Sender<u32>,
    _phantom: PhantomData<T>,
}

impl<T> RenderObjectHandle<T> {
    pub fn new(id: u32, sender: Sender<u32>) -> Self {
        Self {
            id,
            sender,
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for RenderObjectHandle<T> {
    fn drop(&mut self) {
        self.sender.send(self.id);
    }
}

pub struct RenderObject<T> {
    data: T,
}

impl<T> RenderObject<T> {}

pub struct RenderObjectStorage<T> {
    sender: Sender<u32>,
    receiver: Receiver<u32>,
    objects: Slab<T>,
}

impl<T> RenderObjectStorage<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self {
            sender,
            receiver,
            objects: Slab::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn insert(&mut self, obj: T) -> u32 {
        self.sync();
        self.objects.insert(obj) as u32
    }

    pub fn sync(&mut self) {
        for index in self.receiver.try_iter() {
            self.objects.remove(index as usize);
        }
    }
}

pub struct RenderObjectSet<T> {
    storage: RenderObjectStorage<T>,
}

impl<T> RenderObjectSet<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            storage: RenderObjectStorage::new(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn sync(&mut self) {
        self.storage.sync()
    }

    pub fn add(&mut self, obj: T) -> RenderObjectHandle<T> {
        let id = self.storage.insert(obj);
        RenderObjectHandle::<T>::new(id, self.storage.sender.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MeshRenderObject {}

    #[test]
    fn basic() {
        type MeshRenderObjectSet = RenderObjectSet<MeshRenderObject>;

        let mut set = MeshRenderObjectSet::new(1);
        assert_eq!(set.len(), 0);

        let handle = set.add(MeshRenderObject {});
        assert_eq!(set.len(), 1);

        drop(handle);

        set.sync();
        assert_eq!(set.len(), 0);
    }
}
