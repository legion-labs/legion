use atomic_refcell::{AtomicRef, AtomicRefCell};
use bit_set::BitSet;

use lgn_utils::HashMap;

use std::{
    alloc::Layout, any::TypeId, marker::PhantomData, mem::MaybeUninit, ptr::NonNull,
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

//
// RenderObjectKey
//

#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug)]
struct RenderObjectKey {
    type_id: TypeId,
}

impl RenderObjectKey {
    fn new<R>() -> Self
    where
        R: RenderObject,
    {
        Self {
            type_id: TypeId::of::<R>(),
        }
    }
}

///
/// RenderObjectId
///
#[derive(Copy, Eq, PartialEq, Hash, Clone, Debug)]
pub struct RenderObjectId {
    render_object_key: RenderObjectKey,
    index: u32,
    generation: u32,
}

impl Ord for RenderObjectId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.render_object_key
            .cmp(&other.render_object_key)
            .then(self.index.cmp(&other.index))
    }
}

impl PartialOrd for RenderObjectId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

//
//
//

struct RenderObjectStorage {
    item_layout: Layout,
    capacity: usize,
    data: NonNull<u8>,
}

impl RenderObjectStorage {
    fn new(item_layout: Layout) -> Self {
        Self {
            item_layout,
            capacity: 0,
            data: NonNull::dangling(),
        }
    }

    fn with_capacity(item_layout: Layout, initial_capacity: usize) -> Self {
        let mut result = Self::new(item_layout);
        result.resize(initial_capacity);
        result
    }

    #[allow(unsafe_code)]
    fn resize(&mut self, new_capacity: usize) {
        if new_capacity > self.capacity {
            let new_data = unsafe {
                if self.capacity == 0 {
                    std::alloc::alloc(self.array_layout(new_capacity))
                } else {
                    std::alloc::realloc(
                        self.data.as_ptr(),
                        self.array_layout(new_capacity),
                        new_capacity,
                    )
                }
            };
            self.data = NonNull::new(new_data).unwrap();
            self.capacity = new_capacity;
        }
    }

    fn set_value(&mut self, index: usize, value: *const u8) {
        todo!();
    }

    fn get_value(&self, index: usize) -> *const u8 {
        todo!();
    }

    fn get_value_mut(&mut self, index: usize) -> *mut u8 {
        todo!();
    }

    fn array_layout(&self, capacity: usize) -> Layout {
        let align = self.item_layout.align();
        let size = self.item_layout.size();
        let aligned_size = (size + align - 1) & !(align - 1);
        Layout::from_size_align(aligned_size * capacity, align).unwrap()
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
struct RenderObjectSetAllocator {
    render_object_key: RenderObjectKey,
    free_slot_index: AtomicI32,      // updated during sync window
    free_slots: Vec<RenderObjectId>, // updated during sync window
    slots_len: usize,
}

impl RenderObjectSetAllocator {
    fn new(render_object_key: RenderObjectKey) -> Self {
        Self {
            render_object_key,
            free_slot_index: AtomicI32::new(0),
            free_slots: Vec::new(),
            slots_len: 0,
        }
    }

    fn alloc(&self) -> RenderObjectId {
        let prev_free_slot_index = self
            .free_slot_index
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        let free_slot_index = prev_free_slot_index - 1;
        if free_slot_index >= 0 {
            let free_slot_index = usize::try_from(free_slot_index).unwrap();
            self.free_slots[free_slot_index]
        } else {
            let over_len = usize::try_from(-free_slot_index - 1).unwrap();
            let free_slot = self.slots_len + over_len;
            RenderObjectId {
                render_object_key: self.render_object_key,
                index: free_slot as u32,
                generation: 0,
            }
        }
    }
}

///
/// RenderObjectSet
///
struct RenderObjectSet {
    storage: RenderObjectStorage,
    generations: Vec<u32>,
    allocated: BitSet,
    inserted: BitSet,
    updated: BitSet,
    removed: BitSet,
    len: usize,
    capacity: usize,
    drop_fn: unsafe fn(*mut u8),
}

impl RenderObjectSet {
    fn new(item_layout: Layout, capacity: usize, drop_fn: unsafe fn(*mut u8)) -> Self {
        Self {
            storage: RenderObjectStorage::with_capacity(item_layout, capacity),
            generations: Vec::with_capacity(capacity),
            allocated: BitSet::with_capacity(capacity),
            inserted: BitSet::with_capacity(capacity),
            updated: BitSet::with_capacity(capacity),
            removed: BitSet::with_capacity(capacity),
            len: 0,
            capacity,
            drop_fn,
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize {
        self.len
    }

    fn begin_frame(&mut self) {
        self.inserted.clear();
        self.updated.clear();
        self.removed.clear();
    }

    fn insert(&mut self, id: RenderObjectId, render_object: *const u8) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.len);
        assert!(id.generation == self.generations[slot_index]);
        assert!(!self.allocated.contains(slot_index));
        self.storage.set_value(slot_index, render_object);
        self.allocated.insert(slot_index);
        self.inserted.insert(slot_index);
        assert!(!self.updated.contains(slot_index));
        assert!(!self.removed.contains(slot_index));
    }

    #[allow(unsafe_code)]
    fn update(&mut self, id: RenderObjectId, render_object: *const u8) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.len);
        assert!(id.generation == self.generations[slot_index]);
        assert!(self.allocated.contains(slot_index));
        let drop_fn = self.drop_fn;
        unsafe {
            drop_fn(self.storage.get_value_mut(slot_index));
        }
        self.storage.set_value(slot_index, render_object);
        self.updated.insert(slot_index);
        assert!(!self.inserted.contains(slot_index));
        assert!(!self.removed.contains(slot_index));
    }

    #[allow(unsafe_code)]
    fn remove(&mut self, id: RenderObjectId) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.len);
        assert!(id.generation == self.generations[slot_index]);
        let drop_fn = self.drop_fn;
        unsafe {
            drop_fn(self.storage.get_value_mut(slot_index));
        }
        self.allocated.remove(slot_index);
        self.removed.insert(slot_index);
        self.generations[slot_index] += 1;
        assert!(!self.inserted.contains(slot_index));
        assert!(!self.updated.contains(slot_index));
    }
}

//
// RenderObjectAllocator
//

pub struct RenderObjectAllocator<'s, R> {
    allocator: AtomicRef<'s, RenderObjectSetAllocator>,
    phantom: PhantomData<R>,
}

impl<'s, R> RenderObjectAllocator<'s, R>
where
    R: RenderObject,
{
    pub fn alloc(&mut self) -> RenderObjectId {
        self.allocator.alloc()
    }
}

//
// RenderObjectsBuilder
//

#[derive(Default)]
pub struct RenderObjectsBuilder {
    primary_types: HashMap<RenderObjectKey, PrimaryTable>,
    secondary_tables: HashMap<RenderObjectKey, SecondaryTable>,
}

impl RenderObjectsBuilder {
    #[must_use]
    #[allow(unsafe_code)]
    pub fn add_primary_type<R>(mut self) -> Self
    where
        R: RenderObject,
    {
        unsafe fn drop_func<R>(x: *mut u8) {
            x.cast::<R>().drop_in_place();
        }

        let key = RenderObjectKey::new::<R>();
        self.primary_types.insert(
            key,
            PrimaryTable {
                key,
                set: AtomicRefCell::new(RenderObjectSet::new(
                    Layout::new::<R>(),
                    256,
                    drop_func::<R>,
                )),
                allocator: AtomicRefCell::new(RenderObjectSetAllocator::new(key)),
            },
        );
        self
    }

    pub fn finalize(self) -> RenderObjects {
        RenderObjects {
            primary_tables: self.primary_types,
            secondary_tables: self.secondary_tables,
        }
    }
}

//
// PrimaryTable
//

struct PrimaryTable {
    key: RenderObjectKey,
    set: AtomicRefCell<RenderObjectSet>,
    allocator: AtomicRefCell<RenderObjectSetAllocator>,
}

impl PrimaryTable {
    #[allow(unsafe_code)]
    fn sync_update(&mut self) {
        let mut allocator = self.allocator.borrow_mut();
        let mut set = self.set.borrow_mut();

        let free_slot_index = allocator
            .free_slot_index
            .load(std::sync::atomic::Ordering::SeqCst);

        if free_slot_index < 0 {
            let additionnal_slots = usize::try_from(-free_slot_index).unwrap();
            let new_len = set.len + additionnal_slots;
            set.storage.resize(new_len);
            set.generations.resize(new_len, 0);
            set.allocated.reserve_len(new_len);
            set.updated.reserve_len(new_len);
            set.removed.reserve_len(new_len);
            allocator.free_slots.clear();
        } else {
            allocator
                .free_slots
                .truncate(usize::try_from(free_slot_index).unwrap());
        }

        set.removed.iter().for_each(|slot_index| {
            let index = u32::try_from(slot_index).unwrap();
            allocator.free_slots.push(RenderObjectId {
                render_object_key: self.key,
                index,
                generation: set.generations[slot_index],
            });
        });

        allocator.free_slot_index.store(
            i32::try_from(allocator.free_slots.len()).unwrap(),
            std::sync::atomic::Ordering::SeqCst,
        );

        allocator.slots_len = set.len;
    }

    fn begin_frame(&self) {
        let mut set = self.set.borrow_mut();
        set.begin_frame();
    }
}

//
// SecondaryTable
//

struct SecondaryTable {
    key: RenderObjectKey,
    primary_key: RenderObjectKey,
    storage: RenderObjectSet,
}

pub struct RenderObjects {
    primary_tables: HashMap<RenderObjectKey, PrimaryTable>,
    secondary_tables: HashMap<RenderObjectKey, SecondaryTable>,
}

impl RenderObjects {
    pub fn create_allocator<'s, R>(&'s self) -> RenderObjectAllocator<'s, R>
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();

        let allocator = self
            .primary_tables
            .get(&render_object_key)
            .unwrap()
            .allocator
            .borrow();

        RenderObjectAllocator {
            allocator,
            phantom: PhantomData,
        }
    }

    pub fn begin_frame(&self) {
        for (_, primary_table) in self.primary_tables.iter() {
            primary_table.begin_frame();
        }
    }

    pub fn sync_update(&mut self) {
        for (_, primary_table) in self.primary_tables.iter_mut() {
            primary_table.sync_update();
        }
    }

    fn insert<R>(&self, render_object_id: RenderObjectId, data: R)
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        assert_eq!(render_object_id.render_object_key, render_object_key);
        let primary_table = self.primary_tables.get(&render_object_key).unwrap();
        primary_table
            .set
            .borrow_mut()
            .insert(render_object_id, &data as *const R as *const u8);
    }

    fn update<R>(&self, render_object_id: RenderObjectId, data: R)
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        assert_eq!(render_object_id.render_object_key, render_object_key);
        let primary_table = self.primary_tables.get(&render_object_key).unwrap();
        primary_table
            .set
            .borrow_mut()
            .update(render_object_id, &data as *const R as *const u8);
    }

    fn remove(&self, render_object_id: RenderObjectId) {
        let render_object_key = render_object_id.render_object_key;
        let primary_table = self.primary_tables.get(&render_object_key).unwrap();
        primary_table.set.borrow_mut().remove(render_object_id);
    }
}

#[allow(unsafe_code)]
unsafe impl Send for RenderObjects {}

#[allow(unsafe_code)]
unsafe impl Sync for RenderObjects {}

///
/// AddRenderObjectCommand
///
pub struct InsertRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand for InsertRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, render_resources: &super::RenderResources) {
        let set = render_resources.get::<RenderObjects>();
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
        let mut set = render_resources.get::<RenderObjects>();
        set.update(self.render_object_id, self.data);
    }
}

///
/// RemoveRenderObjectCommand
///
pub struct RemoveRenderObjectCommand {
    pub render_object_id: RenderObjectId,
}

impl RenderCommand for RemoveRenderObjectCommand {
    fn execute(self, render_resources: &super::RenderResources) {
        let mut set = render_resources.get::<RenderObjects>();
        set.remove(self.render_object_id);
    }
}
