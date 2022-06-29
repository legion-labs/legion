use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use bit_set::BitSet;

use lgn_utils::HashMap;

#[cfg(debug_assertions)]
use std::any::type_name;
use std::{
    alloc::Layout,
    any::TypeId,
    cell::{Cell, RefCell},
    intrinsics::transmute,
    marker::PhantomData,
    ptr::NonNull,
    sync::{atomic::AtomicI32, Arc},
};

use super::{CommandBuilder, CommandQueuePool, RenderCommand, RenderResources};

//
// RenderObjectId

pub trait RenderObject: 'static + Send {}

impl<T> RenderObject for T where T: 'static + Send {}

//
// RenderObjectKey
//

#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug)]
struct RenderObjectKey {
    type_id: TypeId,
    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl RenderObjectKey {
    fn new<R>() -> Self
    where
        R: RenderObject,
    {
        Self {
            type_id: TypeId::of::<R>(),
            #[cfg(debug_assertions)]
            type_name: type_name::<R>(),
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
// RenderObjectIdPool
//

struct RenderObjectIdPoolInner {
    render_object_key: RenderObjectKey,
    free_slot_index: AtomicI32,
    free_slots: RefCell<Vec<RenderObjectId>>,
    slots_len: Cell<usize>,
}

#[allow(unsafe_code)]
unsafe impl Sync for RenderObjectIdPoolInner {}

#[derive(Clone)]
pub struct RenderObjectIdPool {
    inner: Arc<RenderObjectIdPoolInner>,
}

impl RenderObjectIdPool {
    fn new(render_object_key: RenderObjectKey) -> Self {
        Self {
            inner: Arc::new(RenderObjectIdPoolInner {
                render_object_key,
                free_slot_index: AtomicI32::new(0),
                free_slots: RefCell::new(Vec::new()),
                slots_len: Cell::new(0),
            }),
        }
    }

    pub fn alloc(&self) -> RenderObjectId {
        let prev_free_slot_index = self
            .inner
            .free_slot_index
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        let free_slot_index = prev_free_slot_index - 1;
        if free_slot_index >= 0 {
            let free_slot_index = usize::try_from(free_slot_index).unwrap();
            self.inner.free_slots.borrow()[free_slot_index]
        } else {
            let over_len = usize::try_from(-free_slot_index - 1).unwrap();
            let free_slot = self.inner.slots_len.get() + over_len;
            RenderObjectId {
                render_object_key: self.inner.render_object_key,
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

    fn len(&self) -> usize {
        self.len
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
// RenderObjectsBuilder
//

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FatPtr {
    data: *const (),
    vtable: *const (),
}

#[allow(unsafe_code)]
fn get_fat_ptr<P, S>(data: Box<dyn SecondaryTableHandler<P, S>>) -> FatPtr {
    // Make a FatPtr from the Box. Implicitly forgotten, must be properly dropped in the SecondaryTable drop impl.
    unsafe {
        let p = Box::into_raw(data);
        transmute::<_, FatPtr>(p)
    }
}

#[allow(unsafe_code)]
fn drop_fat_ptr<P, S>(fat_ptr: FatPtr) {
    // Convert back from FatPtr to the actual type, and re-box it so it will be dropped properly.
    unsafe {
        let handler: *mut dyn SecondaryTableHandler<P, S> = transmute(fat_ptr);
        Box::from_raw(handler);
    }
}

#[derive(Default)]
pub struct RenderObjectsBuilder {
    primary_tables: HashMap<RenderObjectKey, (PrimaryTable, PrimaryTableCommandQueuePool)>,
    secondary_tables: HashMap<RenderObjectKey, AtomicRefCell<SecondaryTable>>,
}

impl RenderObjectsBuilder {
    #[allow(unsafe_code)]
    pub fn add_primary_table<P>(&mut self) -> &mut Self
    where
        P: RenderObject,
    {
        unsafe fn drop_func<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place();
        }

        let key = RenderObjectKey::new::<P>();
        self.primary_tables.insert(
            key,
            (
                PrimaryTable {
                    key,
                    set: AtomicRefCell::new(RenderObjectSet::new(
                        Layout::new::<P>(),
                        256,
                        drop_func::<P>,
                    )),
                    render_object_id_pool: RenderObjectIdPool::new(key),
                },
                PrimaryTableCommandQueuePool::new(),
            ),
        );
        self
    }

    #[allow(dead_code)]
    #[allow(unsafe_code)]
    pub fn add_secondary_table<P, S>(&mut self) -> &mut Self
    where
        P: RenderObject,
        S: RenderObject + Default,
    {
        self.add_secondary_table_with_handler::<P, S>(Box::new(
            DefaultSecondaryTableHandler::<P, S>::default(),
        ))
    }

    #[allow(unsafe_code)]
    pub fn add_secondary_table_with_handler<P, S>(
        &mut self,
        handler: Box<dyn SecondaryTableHandler<P, S>>,
    ) -> &mut Self
    where
        P: RenderObject,
        S: RenderObject,
    {
        unsafe fn insert_fn<P, S>(
            h: FatPtr,
            render_resources: &RenderResources,
            render_object_id: RenderObjectId,
            p: *const u8,
            s: *mut u8,
        ) {
            let handler: &dyn SecondaryTableHandler<P, S> = transmute(h);
            let primary_ref = &*p.cast::<P>();
            let result = handler.insert(render_resources, render_object_id, primary_ref);
            s.cast::<S>().write(result);
        }

        unsafe fn update_fn<P, S>(
            h: FatPtr,
            render_resources: &RenderResources,
            render_object_id: RenderObjectId,
            p: *const u8,
            s: *mut u8,
        ) {
            let handler: &dyn SecondaryTableHandler<P, S> = transmute(h);
            let primary_ref = &*p.cast::<P>();
            let secondary_ref = &mut *s.cast::<S>();
            handler.update(
                render_resources,
                render_object_id,
                primary_ref,
                secondary_ref,
            );
        }

        unsafe fn remove_fn<P, S>(
            h: FatPtr,
            render_resources: &RenderResources,
            render_object_id: RenderObjectId,
            p: *const u8,
            s: *mut u8,
        ) {
            let handler: &dyn SecondaryTableHandler<P, S> = transmute(h);
            let primary_ref = &*p.cast::<P>();
            let secondary_ref = &mut *s.cast::<S>();
            handler.remove(
                render_resources,
                render_object_id,
                primary_ref,
                secondary_ref,
            );
        }

        unsafe fn storage_drop_func<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place();
        }

        let primary_key = RenderObjectKey::new::<P>();
        let secondary_key = RenderObjectKey::new::<S>();

        assert!(self.primary_tables.contains_key(&primary_key));

        self.secondary_tables.insert(
            secondary_key,
            AtomicRefCell::new(SecondaryTable {
                _key: secondary_key,
                primary_key,
                storage: RenderObjectStorage::new(Layout::new::<S>(), storage_drop_func::<S>, 256),
                handler_fat_ptr: get_fat_ptr(handler),
                insert_fn: insert_fn::<P, S>,
                update_fn: update_fn::<P, S>,
                remove_fn: remove_fn::<P, S>,
                drop_fn: drop_fat_ptr::<P, S>,
            }),
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
// PrimaryTableCommandPool
//

pub type PrimaryTableCommandQueuePool = CommandQueuePool<PrimaryTable>;
pub type PrimaryTableCommandBuilder = CommandBuilder<PrimaryTable>;

//
// PrimaryTable
//
pub struct PrimaryTable {
    key: RenderObjectKey,
    set: AtomicRefCell<RenderObjectSet>,
    render_object_id_pool: RenderObjectIdPool,
}

impl PrimaryTable {
    #[allow(unsafe_code)]
    fn sync_update(&mut self) {
        let mut set = self.set.borrow_mut();

        let free_slot_index = self
            .render_object_id_pool
            .inner
            .free_slot_index
            .load(std::sync::atomic::Ordering::SeqCst);

        let mut free_slots = self.render_object_id_pool.inner.free_slots.borrow_mut();
        if free_slot_index < 0 {
            let additionnal_slots = usize::try_from(-free_slot_index).unwrap();
            let new_len = set.len + additionnal_slots;
            set.resize(new_len);
            free_slots.clear();
        } else {
            free_slots.truncate(usize::try_from(free_slot_index).unwrap());
        }

        set.removed.iter().for_each(|slot_index| {
            let index = u32::try_from(slot_index).unwrap();
            free_slots.push(RenderObjectId {
                render_object_key: self.key,
                index,
                generation: set.generations[slot_index],
            });
        });

        self.render_object_id_pool.inner.free_slot_index.store(
            i32::try_from(free_slots.len()).unwrap(),
            std::sync::atomic::Ordering::SeqCst,
        );

        self.render_object_id_pool.inner.slots_len.replace(set.len);
    }

    fn begin_frame(&self) {
        {
            let mut set = self.set.borrow_mut();
            set.begin_frame();
        }
    }

    #[allow(unsafe_code)]
    #[allow(dead_code)]
    pub fn try_get<R: RenderObject>(&self, id: RenderObjectId) -> Option<&R> {
        let index = id.index as usize;
        let generation = id.generation;
        if self.set.borrow().allocated.contains(index)
            && self.set.borrow().generations[index] == generation
        {
            unsafe { Some(&*self.set.borrow().storage.get_value(index).cast::<R>()) }
        } else {
            None
        }
    }

    #[allow(unsafe_code)]
    pub fn get<R: RenderObject>(&self, id: RenderObjectId) -> &R {
        let index = id.index as usize;
        let generation = id.generation;

        // To avoid having to duplicate the asserts.
        #[cfg(debug_assertions)]
        let type_name = self.key.type_name;
        #[cfg(not(debug_assertions))]
        let type_name = "unknown";

        assert!(
            self.set.borrow().allocated.contains(index),
            "RenderObject of type {} index {} not allocated.",
            type_name,
            index
        );
        assert!(
            self.set.borrow().generations[index] == generation,
            "RenderObject of type {} index {} generation mismatch (expected {} got {})",
            type_name,
            index,
            self.set.borrow().generations[index],
            generation
        );
        unsafe { &*self.set.borrow().storage.get_value(index).cast::<R>() }
    }
}

//
// PrimaryTableView
//

pub struct PrimaryTableView<R: RenderObject> {
    allocator: RenderObjectIdPool,
    command_queue: PrimaryTableCommandQueuePool,
    _phantom: PhantomData<R>,
}

impl<R: RenderObject> PrimaryTableView<R> {
    pub fn allocate(&self) -> RenderObjectId {
        self.allocator.alloc()
    }

    pub fn command_builder(&self) -> PrimaryTableCommandBuilder {
        self.command_queue.builder()
    }

    #[allow(dead_code)]
    pub fn writer(&self) -> PrimaryTableWriter<'_, R> {
        PrimaryTableWriter {
            view: self,
            command_builder: self.command_queue.builder(),
        }
    }
}

#[allow(unsafe_code)]
unsafe impl<R: RenderObject> Send for PrimaryTableView<R> {}

#[allow(unsafe_code)]
unsafe impl<R: RenderObject> Sync for PrimaryTableView<R> {}

//
// PrimaryTableWriter
//

#[allow(dead_code)]
pub struct PrimaryTableWriter<'a, R: RenderObject> {
    view: &'a PrimaryTableView<R>,
    command_builder: PrimaryTableCommandBuilder,
}

#[allow(dead_code)]
impl<'a, R: RenderObject> PrimaryTableWriter<'a, R> {
    pub fn insert(&mut self, data: R) -> RenderObjectId {
        let render_object_id = self.view.allocate();
        self.command_builder.push(InsertRenderObjectCommand::<R> {
            render_object_id,
            data,
        });
        render_object_id
    }

    pub fn update(&mut self, render_object_id: RenderObjectId, data: R) {
        self.command_builder.push(UpdateRenderObjectCommand::<R> {
            render_object_id,
            data,
        });
    }

    pub fn remove(&mut self, render_object_id: RenderObjectId) {
        self.command_builder
            .push(RemoveRenderObjectCommand { render_object_id });
    }
}

pub trait SecondaryTableHandler<P, S> {
    fn insert(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        render_object: &P,
    ) -> S;
    fn update(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        render_object: &P,
        render_object_private_data: &mut S,
    );
    fn remove(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        render_object: &P,
        render_object_private_data: &mut S,
    );
}

pub struct DefaultSecondaryTableHandler<P, S> {
    phantom: PhantomData<(P, S)>,
}

impl<P, S> Default for DefaultSecondaryTableHandler<P, S> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

impl<P, S> SecondaryTableHandler<P, S> for DefaultSecondaryTableHandler<P, S>
where
    P: RenderObject,
    S: RenderObject + Default,
{
    fn insert(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        _render_object: &P,
    ) -> S {
        S::default()
    }
    fn update(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        _render_object: &P,
        _render_object_private_data: &mut S,
    ) {
    }
    fn remove(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        _render_object: &P,
        _render_object_private_data: &mut S,
    ) {
    }
}

//
// SecondaryTable
//

pub struct SecondaryTable {
    _key: RenderObjectKey,
    primary_key: RenderObjectKey,
    storage: RenderObjectStorage,
    handler_fat_ptr: FatPtr,
    insert_fn:
        unsafe fn(FatPtr, render_resources: &RenderResources, RenderObjectId, *const u8, *mut u8),
    update_fn:
        unsafe fn(FatPtr, render_resources: &RenderResources, RenderObjectId, *const u8, *mut u8),
    remove_fn:
        unsafe fn(FatPtr, render_resources: &RenderResources, RenderObjectId, *const u8, *mut u8),
    drop_fn: unsafe fn(FatPtr),
}

#[allow(unsafe_code)]
impl Drop for SecondaryTable {
    fn drop(&mut self) {
        let drop_fn = self.drop_fn;
        unsafe {
            drop_fn(self.handler_fat_ptr);
        }
    }
}

impl SecondaryTable {
    #[allow(unsafe_code)]
    pub fn get<R: RenderObject>(&self, id: RenderObjectId) -> &R {
        let index = id.index as usize;
        unsafe { &*self.storage.get_value(index).cast::<R>() }
    }

    #[allow(unsafe_code)]
    pub fn get_mut<R: RenderObject>(&mut self, id: RenderObjectId) -> &mut R {
        let index = id.index as usize;
        unsafe { &mut *self.storage.get_value_mut(index).cast::<R>() }
    }
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
        let set = primary_table.0.set.borrow();
        Self {
            set,
            // render_objects,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> RenderObjectQueryIter<'_, R> {
        RenderObjectQueryIter::new(self)
    }

    #[allow(unsafe_code)]
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(usize, &R),
    {
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
    primary_tables: HashMap<RenderObjectKey, (PrimaryTable, PrimaryTableCommandQueuePool)>,
    secondary_tables: HashMap<RenderObjectKey, AtomicRefCell<SecondaryTable>>,
}

impl RenderObjects {
    pub fn begin_frame(&self, render_resources: &RenderResources) {
        // Remove private data for RenderObjects removed last frame, before clearing the removed items (in begin_frame below).
        for secondary_table in self.secondary_tables.values() {
            let mut secondary_table = secondary_table.borrow_mut();

            let primary_table_key = secondary_table.primary_key;
            let primary_table = &self.primary_tables.get(&primary_table_key).unwrap().0;
            let primary_table_set = primary_table.set.borrow();

            let handler_fat_ptr = secondary_table.handler_fat_ptr;

            for removed_index in primary_table_set.removed.iter() {
                let remove_fn = secondary_table.remove_fn;
                let render_object_id = RenderObjectId {
                    render_object_key: primary_table_key,
                    index: removed_index as u32,
                    generation: primary_table_set.generations[removed_index],
                };
                let primary_ref = primary_table_set.storage.get_value(removed_index);
                let secondary_ref = secondary_table.storage.get_value_mut(removed_index);
                #[allow(unsafe_code)]
                unsafe {
                    remove_fn(
                        handler_fat_ptr,
                        render_resources,
                        render_object_id,
                        primary_ref,
                        secondary_ref,
                    );
                }
                secondary_table.storage.remove_value(removed_index);
            }
        }

        for (_, (primary_table, command_queue)) in self.primary_tables.iter() {
            primary_table.begin_frame();
            command_queue.apply(primary_table);
        }

        // Add / update private data for RenderObjects which were added / updated this frame.
        for secondary_table in self.secondary_tables.values() {
            let mut secondary_table = secondary_table.borrow_mut();

            let primary_table_key = secondary_table.primary_key;
            let primary_table = &self.primary_tables.get(&primary_table_key).unwrap().0;
            let primary_table_set = primary_table.set.borrow();

            secondary_table.storage.resize(primary_table_set.len());

            let handler_fat_ptr = secondary_table.handler_fat_ptr;

            for inserted_index in primary_table_set.inserted.iter() {
                let insert_fn = secondary_table.insert_fn;
                let render_object_id = RenderObjectId {
                    render_object_key: primary_table_key,
                    index: inserted_index as u32,
                    generation: primary_table_set.generations[inserted_index],
                };
                let primary_ref = primary_table_set.storage.get_value(inserted_index);
                let secondary_ref = secondary_table.storage.get_value_mut(inserted_index);
                #[allow(unsafe_code)]
                unsafe {
                    insert_fn(
                        handler_fat_ptr,
                        render_resources,
                        render_object_id,
                        primary_ref,
                        secondary_ref,
                    );
                }
            }

            for updated_index in primary_table_set.updated.iter() {
                let update_fn = secondary_table.update_fn;
                let render_object_id = RenderObjectId {
                    render_object_key: primary_table_key,
                    index: updated_index as u32,
                    generation: primary_table_set.generations[updated_index],
                };
                let primary_ref = primary_table_set.storage.get_value(updated_index);
                let secondary_ref = secondary_table.storage.get_value_mut(updated_index);
                #[allow(unsafe_code)]
                unsafe {
                    update_fn(
                        handler_fat_ptr,
                        render_resources,
                        render_object_id,
                        primary_ref,
                        secondary_ref,
                    );
                }
            }
        }
    }

    pub fn sync_update(&mut self) {
        for (_, (primary_table, command_queue)) in self.primary_tables.iter_mut() {
            primary_table.sync_update();
            command_queue.sync_update();
        }
    }

    pub fn primary_table_view<R>(&self) -> PrimaryTableView<R>
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        let primary_table_and_queue = self.primary_tables.get(&render_object_key).unwrap();

        PrimaryTableView {
            allocator: primary_table_and_queue.0.render_object_id_pool.clone(),
            command_queue: primary_table_and_queue.1.clone(),
            _phantom: PhantomData,
        }
    }

    pub fn primary_table<R>(&self) -> &PrimaryTable
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        &self.primary_tables.get(&render_object_key).unwrap().0
    }

    pub fn secondary_table_mut<R>(&self) -> AtomicRefMut<'_, SecondaryTable>
    where
        R: RenderObject,
    {
        let render_object_key = RenderObjectKey::new::<R>();
        self.secondary_tables
            .get(&render_object_key)
            .unwrap()
            .borrow_mut()
    }
}

#[allow(unsafe_code)]
unsafe impl Send for RenderObjects {}

#[allow(unsafe_code)]
unsafe impl Sync for RenderObjects {}

impl Drop for RenderObjects {
    fn drop(&mut self) {
        for secondary_table in self.secondary_tables.values() {
            let mut secondary_table = secondary_table.borrow_mut();

            let primary_table_key = secondary_table.primary_key;
            let primary_table = &self.primary_tables.get(&primary_table_key).unwrap().0;
            let primary_table_set = primary_table.set.borrow();

            for allocated_index in primary_table_set.allocated.iter() {
                secondary_table.storage.remove_value(allocated_index);
            }
        }

        for (_, (primary_table, _)) in self.primary_tables.iter_mut() {
            let mut primary_table_set = primary_table.set.borrow_mut();
            let allocated = primary_table_set.allocated.clone();
            for allocated_index in allocated.iter() {
                primary_table_set.storage.remove_value(allocated_index);
            }
        }
    }
}
//
// AddRenderObjectCommand
//
pub struct InsertRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand<PrimaryTable> for InsertRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, primary_table: &PrimaryTable) {
        primary_table.set.borrow_mut().insert(
            self.render_object_id,
            std::ptr::addr_of!(self.data).cast::<u8>(),
        );
        std::mem::forget(self.data);
    }
}
//
// UpdateRenderObjectCommand
//
pub struct UpdateRenderObjectCommand<R> {
    pub render_object_id: RenderObjectId,
    pub data: R,
}

impl<R> RenderCommand<PrimaryTable> for UpdateRenderObjectCommand<R>
where
    R: RenderObject,
{
    fn execute(self, primary_table: &PrimaryTable) {
        primary_table.set.borrow_mut().update(
            self.render_object_id,
            std::ptr::addr_of!(self.data).cast::<u8>(),
        );
        std::mem::forget(self.data);
    }
}

//
// RemoveRenderObjectCommand
//
pub struct RemoveRenderObjectCommand {
    pub render_object_id: RenderObjectId,
}

impl RenderCommand<PrimaryTable> for RemoveRenderObjectCommand {
    fn execute(self, primary_table: &PrimaryTable) {
        primary_table.set.borrow_mut().remove(self.render_object_id);
    }
}
