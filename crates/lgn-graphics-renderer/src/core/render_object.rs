use bit_set::BitSet;
use crossbeam_channel::{Receiver, Sender};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    sync::atomic::AtomicI32,
};

use super::RenderCommand;

///
/// RenderObjectId
///
pub trait AsRenderObject<R>
where
    R: RenderObject,
{
    fn as_render_object(&self) -> R;
}

pub trait RenderObject: 'static + Send {}

impl<T> RenderObject for T where T: 'static + Send {}

///
/// RenderObjectId
///
#[derive(Copy, Eq, PartialEq, Hash, Clone, Debug)]
pub struct RenderObjectId {
    feature_idx: u32,
    index: u32,
    generation: u32,
}

const RENDER_OBJECT_INVALID: RenderObjectId = RenderObjectId {
    feature_idx: u32::MAX,
    index: u32::MAX,
    generation: u32::MAX,
};

impl RenderObjectId {
    pub fn is_valid(&self) -> bool {
        *self != RENDER_OBJECT_INVALID
    }
}

impl Ord for RenderObjectId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.feature_idx
            .cmp(&other.feature_idx)
            .then(self.index.cmp(&other.index))
    }
}

impl PartialOrd for RenderObjectId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for RenderObjectId {
    fn default() -> Self {
        RENDER_OBJECT_INVALID
    }
}

///
/// Slot
///
struct Slot<R> {
    value: MaybeUninit<R>,
    generation: u32,
}

impl<R> Default for Slot<R> {
    fn default() -> Self {
        Self {
            value: MaybeUninit::uninit(),
            generation: 0,
        }
    }
}

///
/// RenderObjectSetAllocator
///
pub struct RenderObjectSetAllocator<R> {
    free_slot_index: AtomicI32, // updated during sync window
    free_slots: Vec<u32>,       // updated during sync window
    _phantom: PhantomData<R>,
}

impl<R> RenderObjectSetAllocator<R>
where
    R: RenderObject,
{
    pub fn new() -> Self {
        Self {
            free_slot_index: AtomicI32::new(0),
            free_slots: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn alloc(&self) -> RenderObjectId {
        let prev_free_slot_index = self
            .free_slot_index
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        let free_slot_index = prev_free_slot_index - 1;
        if free_slot_index >= 0 {
            let free_slot_index = usize::try_from(free_slot_index).unwrap();
            let free_slot = self.free_slots[free_slot_index];
            RenderObjectId {
                feature_idx: 0,
                index: free_slot,
                generation: self.slots[free_slot as usize].generation,
            }
        } else {
            let over_len = usize::try_from(-free_slot_index - 1).unwrap();
            let free_slot = self.slots.len() + over_len;
            RenderObjectId {
                feature_idx: 0,
                index: free_slot as u32,
                generation: 0,
            }
        }
    }
}

///
/// RenderObjectSet
///
pub struct RenderObjectSet<R> {
    slots: Vec<Slot<R>>,
    allocated: BitSet,
    inserted: BitSet,
    updated: BitSet,
    removed: BitSet,
}

impl<R> RenderObjectSet<R>
where
    R: RenderObject,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            allocated: BitSet::with_capacity(capacity),
            inserted: BitSet::with_capacity(capacity),
            updated: BitSet::with_capacity(capacity),
            removed: BitSet::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[allow(unsafe_code)]
    pub fn sync_update(&mut self, allocator: &mut RenderObjectSetAllocator<R>) {
        self.resize_containers(allocator);

        self.removed
            .iter()
            .for_each(|slot_index| self.free_slots.push(u32::try_from(slot_index).unwrap()));

        self.free_slot_index.store(
            i32::try_from(self.free_slots.len()).unwrap(),
            std::sync::atomic::Ordering::SeqCst,
        );
    }

    pub fn begin_frame(&mut self) {
        self.inserted.clear();
        self.updated.clear();
        self.removed.clear();
    }

    pub fn insert(&mut self, id: RenderObjectId, render_object: R) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.slots.len());
        let slot = &mut self.slots[slot_index];
        assert!(id.generation == slot.generation);
        assert!(!self.allocated.contains(slot_index));
        slot.value = MaybeUninit::new(render_object);
        self.allocated.insert(slot_index);
        self.inserted.insert(slot_index);
        assert!(!self.updated.contains(slot_index));
        assert!(!self.removed.contains(slot_index));
    }

    #[allow(unsafe_code)]
    pub fn update(&mut self, id: RenderObjectId, render_object: R) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.slots.len());
        let slot = &mut self.slots[slot_index];
        assert!(id.generation == slot.generation);
        assert!(self.allocated.contains(slot_index));
        unsafe {
            slot.value.assume_init_drop();
        }
        slot.value = MaybeUninit::new(render_object);
        self.updated.insert(slot_index);
        assert!(!self.inserted.contains(slot_index));
        assert!(!self.removed.contains(slot_index));
    }

    #[allow(unsafe_code)]
    pub fn remove(&mut self, id: RenderObjectId) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.slots.len());
        let slot = &mut self.slots[slot_index];
        assert!(id.generation == slot.generation);
        unsafe {
            slot.value.assume_init_drop();
        }
        self.allocated.remove(slot_index);
        self.removed.insert(slot_index);
        assert!(!self.inserted.contains(slot_index));
        assert!(!self.updated.contains(slot_index));
    }

    #[allow(unsafe_code)]
    fn resize_containers(&mut self, allocator: &RenderObjectSetAllocator<R>) {
        let free_slot_index = allocator
            .free_slot_index
            .load(std::sync::atomic::Ordering::SeqCst);
        if free_slot_index < 0 {
            let additionnal_slots = usize::try_from(-free_slot_index).unwrap();
            let new_len = self.slots.len() + additionnal_slots;
            self.slots.reserve(additionnal_slots);
            unsafe {
                self.slots.set_len(new_len);
            }
            self.allocated.reserve_len(new_len);
            self.updated.reserve_len(new_len);
            self.removed.reserve_len(new_len);
        }
    }
}

///
/// AddRenderObjectCommand
///
pub struct AddRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand for AddRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, render_resources: &super::RenderResources) {
        let mut set = render_resources.get_mut::<RenderObjectSet<R>>();
        set.insert(self.render_object_id, self.data);
    }
}

///
/// UpdateRenderObjectCommand
///
pub struct UpdateRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand for UpdateRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, render_resources: &super::RenderResources) {
        let mut set = render_resources.get_mut::<RenderObjectSet<R>>();
        set.update(self.render_object_id, self.data);
    }
}
