#![allow(unsafe_code)]

use std::alloc::Layout;
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::forget;
use std::ptr::{null, NonNull};

use anyhow::{anyhow, Result};

/**
 * Object unique ID model wide
 **/
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ModelKey(u64);

impl ModelKey {
    fn new(s: &str) -> Self {
        let mut hasher = DefaultHasher::default();
        s.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/**
 * Per-type container
 **/
#[derive(Debug)]
struct ModelVec {
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: NonNull<u8>,
    drop_fn: unsafe fn(*mut u8),
}

impl ModelVec {
    fn new(item_layout: Layout, drop_fn: unsafe fn(*mut u8)) -> Self {
        Self {
            item_layout,
            capacity: 0,
            len: 0,
            data: NonNull::dangling(),
            drop_fn,
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn data(&self) -> NonNull<u8> {
        self.data
    }

    fn add(&mut self, value: *const u8) -> usize {
        self.reserve(1);
        let index = self.len;
        let ptr = self.get_unchecked(index);
        unsafe {
            std::ptr::copy_nonoverlapping(value, ptr, self.item_layout.size());
        }
        self.len += 1;
        index
    }

    fn get_object_ref(&self, index: usize) -> *const u8 {
        assert!(index < self.size());
        self.get_unchecked(index)
    }

    fn reserve(&mut self, additionnal: usize) {
        assert!(additionnal > 0);

        let needed_capacity = self.len + additionnal;
        if needed_capacity > self.capacity {
            let additionnal = needed_capacity - self.capacity;
            let additionnal = (additionnal + 1024 - 1) & !(1024 - 1);
            self.grow(additionnal);
        }

        assert!(self.len + additionnal <= self.capacity);
    }

    fn grow(&mut self, additionnal: usize) {
        assert!(additionnal > 0);

        let new_capacity = self.capacity + additionnal;
        let new_layout = array_layout(&self.item_layout, new_capacity);
        let new_data = unsafe {
            if self.capacity == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(
                    self.data.as_ptr(),
                    array_layout(&self.item_layout, self.capacity),
                    new_capacity,
                )
            }
        };
        self.data = NonNull::new(new_data).unwrap();
        self.capacity = new_capacity;
    }

    fn get_unchecked(&self, index: usize) -> *mut u8 {
        assert!(index < self.capacity);
        let ptr = self.data.as_ptr();
        unsafe { ptr.add(index * self.item_layout.size()) }
    }
}

impl Drop for ModelVec {
    fn drop(&mut self) {
        let drop_fn = self.drop_fn;
        for i in 0..self.size() {
            unsafe {
                drop_fn(self.get_unchecked(i));
            }
        }
    }
}

fn array_layout(item_layout: &Layout, capacity: usize) -> Layout {
    let align = item_layout.align();
    let size = item_layout.size();
    let aligned_size = (size + align - 1) & !(align - 1);
    Layout::from_size_align(aligned_size * capacity, item_layout.align()).unwrap()
}

/**
 * General properties for code generation.
 **/
pub trait ModelObject: 'static + Clone + Sized + Hash + PartialEq {
    fn typename() -> &'static str;
    fn name(&self) -> &str;
}

/**
 * Typed handle
**/
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ModelHandle<T>
where
    T: ModelObject,
{
    id: u32,
    _phantom: PhantomData<*const T>,
}

impl<T> ModelHandle<T>
where
    T: ModelObject,
{
    fn new(id: u32) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    // pub fn get_ref<'model>(&self, model: &'model Model) -> ModelRef<'model, T> {
    //     ModelRef {
    //         id: self.id,
    //         object: model.get_from_id(self.id).unwrap(),
    //     }
    // }

    pub fn get<'model>(&self, model: &'model Model) -> &'model T {
        model.get_from_id(self.id).unwrap()
    }

    pub fn id(self) -> u32 {
        self.id
    }
}

impl<T> Copy for ModelHandle<T> where T: ModelObject {}

/**
 * Helper to reference an object inside the model
 **/
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct PairTypeId {
    type_index: u32,
    object_index: u32,
}

impl PairTypeId {
    fn new(type_index: u32, object_index: u32) -> Self {
        Self {
            type_index,
            object_index,
        }
    }
}

/**
 * Typed ref
 **/

#[derive(Clone, Copy)]
pub struct ModelRef<'a, T>
where
    T: ModelObject,
{
    id: u32,
    object: &'a T,
}

impl<'a, T> ModelRef<'a, T>
where
    T: ModelObject,
{
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn object(&self) -> &'a T {
        self.object
    }
}

/**
 * Iterator
 **/
pub struct ModelVecIter<'a, T: ModelObject> {
    index: u32,
    size: u32,
    start_ptr: *const T,
    _marker: PhantomData<&'a ModelVec>,
}

impl<'a, T: ModelObject> Default for ModelVecIter<'a, T> {
    fn default() -> Self {
        Self {
            index: 0,
            size: 0,
            start_ptr: null(),
            _marker: PhantomData::default(),
        }
    }
}

impl<'a, T: ModelObject> ModelVecIter<'a, T> {
    fn new(model_vec: Option<&'a ModelVec>) -> Self {
        if let Some(model_vec) = model_vec {
            Self {
                index: 0,
                size: u32::try_from(model_vec.size()).unwrap(),
                start_ptr: model_vec.data().cast::<T>().as_ptr(),
                _marker: PhantomData::default(),
            }
        } else {
            Self::default()
        }
    }
}

impl<'a, T: ModelObject> Iterator for ModelVecIter<'a, T> {
    type Item = ModelRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let id = self.index;
            let start_ptr = self.start_ptr;
            let cur_ptr = unsafe { start_ptr.add(id as usize) };
            let cur_ref = unsafe { &*cur_ptr };

            self.index += 1;

            return Some(ModelRef {
                id,
                object: cur_ref,
            });
        }
        None
    }
}

/**
 * Interface for creating and managing objects
 **/
#[derive(Debug, Default)]
pub struct Model {
    model_vecs: Vec<ModelVec>,
    type_map: HashMap<TypeId, usize, fxhash::FxBuildHasher>,
    key_map: HashMap<ModelKey, PairTypeId, fxhash::FxBuildHasher>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size<T: ModelObject>(&self) -> usize {
        match self.get_container::<T>() {
            Some(e) => e.size(),
            None => 0,
        }
    }

    /// Add an object to the model.
    ///
    /// # Errors
    /// todo
    pub fn add<T: ModelObject>(&mut self, key: &str, value: T) -> Result<ModelHandle<T>> {
        let key = ModelKey::new(key);
        if self.key_map.contains_key(&key) {
            return Err(anyhow!("Object not unique"));
        }
        let type_index = self.get_or_create_container::<T>();
        let value_ptr = (&value as *const T).cast::<u8>();
        let object_index = self.get_container_by_index_mut(type_index).add(value_ptr);
        forget(value);
        let object_index = u32::try_from(object_index).unwrap();
        let object_id = PairTypeId::new(u32::try_from(type_index).unwrap(), object_index);
        self.key_map.insert(key, object_id);

        Ok(ModelHandle::new(object_index))
    }

    pub fn object_iter<T: ModelObject>(&self) -> ModelVecIter<'_, T> {
        let container = self.get_container::<T>();
        ModelVecIter::new(container)
    }

    pub fn get_object_handle<T: ModelObject>(&self, key: &str) -> Option<ModelHandle<T>> {
        let container_index = self.get_container_index::<T>()?;
        let key = ModelKey::new(key);
        let id = self.key_map.get(&key).copied()?;
        assert!(id.type_index as usize == container_index);
        Some(ModelHandle::new(id.object_index))
    }

    pub fn get_from_id<T: ModelObject>(&self, id: u32) -> Option<&T> {
        let container = self.get_container::<T>()?;
        let ptr = container.get_object_ref(id as usize).cast::<T>();
        unsafe { ptr.as_ref() }
    }

    fn get_or_create_container<T: ModelObject>(&mut self) -> usize {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place();
        }
        let type_id = TypeId::of::<T>();
        let type_index = self.type_map.entry(type_id).or_insert_with(|| {
            let index = self.model_vecs.len();
            self.model_vecs
                .push(ModelVec::new(Layout::new::<T>(), drop_ptr::<T>));
            index
        });

        *type_index
    }

    fn get_container_index<T: ModelObject>(&self) -> Option<usize> {
        let type_id = TypeId::of::<T>();
        self.type_map.get(&type_id).copied()
    }

    fn get_container<T: ModelObject>(&self) -> Option<&ModelVec> {
        let index = self.get_container_index::<T>()?;
        Some(self.get_container_by_index(index))
    }

    fn get_container_by_index(&self, index: usize) -> &ModelVec {
        &self.model_vecs[index]
    }

    fn get_container_by_index_mut(&mut self, index: usize) -> &mut ModelVec {
        &mut self.model_vecs[index]
    }
}
