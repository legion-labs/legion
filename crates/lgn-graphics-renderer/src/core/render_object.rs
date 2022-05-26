use atomic_refcell::{AtomicRef, AtomicRefCell};
use bit_set::BitSet;

use lgn_transform::prelude::GlobalTransform;
use lgn_utils::HashMap;

use std::{alloc::Layout, any::TypeId, marker::PhantomData, ptr::NonNull, sync::atomic::AtomicI32};

use super::RenderCommand;

//
// RenderObjectId
//
pub trait AsSpatialRenderObject<R>
where
    R: RenderObject,
{
    fn as_spatial_render_object(&self, transform: GlobalTransform) -> R;
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

//
// RenderObjectId
//
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
// RenderObjectStorage
//

struct RenderObjectStorage {
    item_layout: Layout,
    capacity: usize,
    data: NonNull<u8>,
    drop_fn: unsafe fn(*mut u8),
}

impl RenderObjectStorage {
    fn new(item_layout: Layout, drop_fn: unsafe fn(*mut u8), initial_capacity: usize) -> Self {
        let mut result = Self {
            item_layout,
            capacity: 0,
            data: NonNull::dangling(),
            drop_fn,
        };
        result.resize(initial_capacity);
        result
    }

    fn get_base_ptr(&self) -> *const u8 {
        self.get_value(0)
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

    #[allow(unsafe_code)]
    fn insert_value(&mut self, index: usize, value: *const u8) {
        let byte_offset = index * self.aligned_size();
        unsafe {
            std::ptr::copy_nonoverlapping(
                value,
                self.data.as_ptr().add(byte_offset),
                self.item_layout.size(),
            );
        }
    }

    fn update_value(&mut self, index: usize, value: *const u8) {
        self.remove_value(index);
        self.insert_value(index, value);
    }

    #[allow(unsafe_code)]
    fn remove_value(&mut self, index: usize) {
        let drop_fn = self.drop_fn;
        unsafe {
            drop_fn(self.get_value_mut(index));
        }
    }

    #[allow(unsafe_code)]
    fn get_value(&self, index: usize) -> *const u8 {
        let byte_offset = index * self.aligned_size();
        unsafe { self.data.as_ptr().add(byte_offset) as *const u8 }
    }

    #[allow(unsafe_code)]
    fn get_value_mut(&mut self, index: usize) -> *mut u8 {
        let byte_offset = index * self.aligned_size();
        unsafe { self.data.as_ptr().add(byte_offset) }
    }

    fn aligned_size(&self) -> usize {
        let align = self.item_layout.align();
        let size = self.item_layout.size();
        (size + align - 1) & !(align - 1)
    }

    fn array_layout(&self, capacity: usize) -> Layout {
        let align = self.item_layout.align();
        let aligned_size = self.aligned_size();
        Layout::from_size_align(aligned_size * capacity, align).unwrap()
    }
}

//
// RenderObjectSetAllocator
//
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

//
// RenderObjectSet
//
struct RenderObjectSet {
    len: usize,
    storage: RenderObjectStorage,
    generations: Vec<u32>,
    allocated: BitSet,
    inserted: BitSet,
    updated: BitSet,
    removed: BitSet,
}

impl RenderObjectSet {
    fn new(item_layout: Layout, capacity: usize, drop_fn: unsafe fn(*mut u8)) -> Self {
        Self {
            len: 0,
            storage: RenderObjectStorage::new(item_layout, drop_fn, capacity),
            generations: Vec::with_capacity(capacity),
            allocated: BitSet::with_capacity(capacity),
            inserted: BitSet::with_capacity(capacity),
            updated: BitSet::with_capacity(capacity),
            removed: BitSet::with_capacity(capacity),
        }
    }

    fn begin_frame(&mut self) {
        self.inserted.clear();
        self.updated.clear();
        self.removed.clear();
    }

    fn resize(&mut self, new_len: usize) {
        self.storage.resize(new_len);
        self.generations.resize(new_len, 0);
        self.allocated.reserve_len_exact(new_len);
        self.updated.reserve_len_exact(new_len);
        self.removed.reserve_len_exact(new_len);
        self.len = new_len;
    }

    fn insert(&mut self, id: RenderObjectId, render_object: *const u8) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.len);
        assert!(id.generation == self.generations[slot_index]);
        assert!(!self.allocated.contains(slot_index));
        self.storage.insert_value(slot_index, render_object);
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
        self.storage.update_value(slot_index, render_object);
        self.updated.insert(slot_index);
        assert!(!self.inserted.contains(slot_index));
        assert!(!self.removed.contains(slot_index));
    }

    #[allow(unsafe_code)]
    fn remove(&mut self, id: RenderObjectId) {
        let slot_index = id.index as usize;
        assert!(slot_index < self.len);
        assert!(id.generation == self.generations[slot_index]);
        self.storage.remove_value(slot_index);
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
    primary_tables: HashMap<RenderObjectKey, PrimaryTable>,
    secondary_tables: HashMap<RenderObjectKey, SecondaryTable>,
}

impl RenderObjectsBuilder {
    #[must_use]
    #[allow(unsafe_code)]
    pub fn add_primary_table<P>(mut self) -> Self
    where
        P: RenderObject,
    {
        unsafe fn drop_func<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place();
        }

        let key = RenderObjectKey::new::<P>();
        self.primary_tables.insert(
            key,
            PrimaryTable {
                key,
                set: AtomicRefCell::new(RenderObjectSet::new(
                    Layout::new::<P>(),
                    256,
                    drop_func::<P>,
                )),
                allocator: AtomicRefCell::new(RenderObjectSetAllocator::new(key)),
            },
        );
        self
    }

    #[must_use]
    #[allow(unsafe_code)]
    pub fn add_secondary_table<P, S>(mut self) -> Self
    where
        P: RenderObject,
        S: RenderObject,
    {
        unsafe fn drop_func<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place();
        }

        let primary_key = RenderObjectKey::new::<P>();
        let secondary_key = RenderObjectKey::new::<S>();

        assert!(self.primary_tables.contains_key(&primary_key));

        self.secondary_tables.insert(
            secondary_key,
            SecondaryTable {
                key: secondary_key,
                primary_key,
                storage: AtomicRefCell::new(RenderObjectStorage::new(
                    Layout::new::<S>(),
                    drop_func::<S>,
                    256,
                )),
            },
        );

        self
    }

    pub fn finalize(self) -> RenderObjects {
        RenderObjects {
            primary_tables: self.primary_tables,
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
            set.resize(new_len);
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
    storage: AtomicRefCell<RenderObjectStorage>,
}

//
// RenderObjectQueryIter
//
pub struct RenderObjectQueryIter<'a, R> {
    storage_ptr: *const R,
    iter: bit_set::Iter<'a, u32>,
    phantom: PhantomData<R>,
}

impl<'a, R> RenderObjectQueryIter<'a, R> {
    fn new(query: &'a RenderObjectQuery<'a, R>) -> Self {
        let storage_ptr = query.set.storage.get_base_ptr().cast::<R>();
        let iter = query.set.allocated.iter();
        // fn new(storage_ptr: *const R, iter: bit_set::Iter<'a, u32>) -> Self {
        Self {
            storage_ptr,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, R> Iterator for RenderObjectQueryIter<'a, R>
where
    R: 'a,
{
    type Item = &'a R;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<&'a R> {
        unsafe { self.iter.next().map(|index| &*self.storage_ptr.add(index)) }
    }
}

//
// RenderObjectQuery
//

pub struct RenderObjectQuery<'a, R> {
    set: AtomicRef<'a, RenderObjectSet>,
    // .. render_objects: &'a RenderObjects,
    phantom: PhantomData<R>,
}

impl<'a, R> RenderObjectQuery<'a, R>
where
    R: RenderObject,
{
    pub fn new(render_objects: &'a RenderObjects) -> Self {
        let render_object_key = RenderObjectKey::new::<R>();
        let primary_table = render_objects
            .primary_tables
            .get(&render_object_key)
            .unwrap();
        let set = primary_table.set.borrow();
        Self {
            set,
            // render_objects,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> RenderObjectQueryIter<'_, R> {
        // let primary_table = self.render_objects.primary_table::<R>();
        // let storage = &primary_table.set.borrow().storage;
        // let iter = primary_table.set.borrow().allocated.iter();
        // RenderObjectQueryIter::new(storage.get_base_ptr().cast::<R>(), iter)
        RenderObjectQueryIter::new(self)
    }

    #[allow(unsafe_code)]
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(usize, &R),
    {
        // let primary_table = self.render_objects.primary_table::<R>();
        // let set = self.set.allocated
        unsafe {
            for index in &self.set.allocated {
                f(index, &*(self.set.storage.get_value(index).cast::<R>()));
            }
        }
    }
}

//
// RenderObjects
//

pub struct RenderObjects {
    primary_tables: HashMap<RenderObjectKey, PrimaryTable>,
    secondary_tables: HashMap<RenderObjectKey, SecondaryTable>,
}

impl RenderObjects {
    pub fn create_allocator<R>(&self) -> RenderObjectAllocator<'_, R>
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

    fn primary_table<R>(&self) -> &PrimaryTable
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        self.primary_tables.get(&render_object_key).unwrap()
    }

    fn insert<R>(&self, render_object_id: RenderObjectId, data: R)
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        assert_eq!(render_object_id.render_object_key, render_object_key);
        let primary_table = self.primary_table::<R>();
        primary_table
            .set
            .borrow_mut()
            .insert(render_object_id, std::ptr::addr_of!(data).cast::<u8>());
        std::mem::forget(data);
    }

    fn update<R>(&self, render_object_id: RenderObjectId, data: R)
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        assert_eq!(render_object_id.render_object_key, render_object_key);
        let primary_table = self.primary_table::<R>();
        primary_table
            .set
            .borrow_mut()
            .update(render_object_id, std::ptr::addr_of!(data).cast::<u8>());
        std::mem::forget(data);
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

//
// AddRenderObjectCommand
//
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
//
// UpdateRenderObjectCommand
//
pub struct UpdateRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand for UpdateRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, render_resources: &super::RenderResources) {
        let set = render_resources.get::<RenderObjects>();
        set.update(self.render_object_id, self.data);
    }
}

//
// RemoveRenderObjectCommand
//
pub struct RemoveRenderObjectCommand {
    pub render_object_id: RenderObjectId,
}

impl RenderCommand for RemoveRenderObjectCommand {
    fn execute(self, render_resources: &super::RenderResources) {
        let set = render_resources.get::<RenderObjects>();
        set.remove(self.render_object_id);
    }
}
