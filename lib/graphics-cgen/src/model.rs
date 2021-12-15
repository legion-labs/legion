#![allow(unsafe_code)]

use anyhow::{anyhow, Result};
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;

use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use std::mem::forget;
use std::ptr::{null, NonNull};
use strum::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ModelKey(u64);

impl From<&str> for ModelKey {
    fn from(s: &str) -> Self {
        let mut hasher = DefaultHasher::default();
        s.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug)]
pub struct ModelVec {
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

pub trait ModelObject: 'static + Clone + Sized {
    fn typename() -> &'static str;
    fn name(&self) -> &str;
    fn key(&self) -> ModelKey {
        ModelKey::from(self.name())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ModelObjectId {
    type_index: u32,
    object_index: u32,
}

impl ModelObjectId {
    fn new(type_index: u32, object_index: u32) -> Self {
        Self {
            type_index,
            object_index,
        }
    }

    pub fn object_id(&self) -> u32 {
        self.object_index
    }
}

// #[derive(Debug, PartialEq, Clone, Copy)]
// pub struct ModelHandle<T: ModelObject> {
//     id: ModelObjectId,
//     _phantom: PhantomData<T>,
// }

// impl<T: ModelObject> Eq for ModelHandle<T> {

// }

// impl<T: ModelObject> Hash for ModelHandle<T> {
//     fn hash<H: Hasher>(&self, mut state: &mut H) {
//         self.id.hash(state)
//     }
// }

// impl<T: ModelObject> ModelHandle<T> {
//     fn new(type_index: u32, object_index: u32) -> Self {
//         Self {
//             id: ModelObjectId::new(type_index, object_index),
//             _phantom: PhantomData::default(),
//         }
//     }

//     pub fn object_id(&self) -> u32 {
//         self.id.object_index
//     }
// }

#[derive(Debug, Default)]
pub struct Model {
    model_vecs: Vec<ModelVec>,
    type_map: HashMap<TypeId, usize, fxhash::FxBuildHasher>,
    key_map: HashMap<ModelKey, ModelObjectId, fxhash::FxBuildHasher>,
}

pub struct ModelVecIter<'a, T: ModelObject> {
    cur_ptr: *const T,
    end_ptr: *const T,
    _marker: PhantomData<&'a ModelVec>,
}

impl<'a, T: ModelObject> Default for ModelVecIter<'a, T> {
    fn default() -> Self {
        Self {
            cur_ptr: null(),
            end_ptr: null(),
            _marker: PhantomData::default(),
        }
    }
}

impl<'a, T: ModelObject> ModelVecIter<'a, T> {
    fn new(model_vec: Option<&'a ModelVec>) -> Self {
        if let Some(model_vec) = model_vec {
            let cur_ptr = model_vec.data().cast::<T>().as_ptr();
            let end_ptr = unsafe { cur_ptr.add(model_vec.size()) };
            Self {
                cur_ptr,
                end_ptr,
                _marker: PhantomData::default(),
            }
        } else {
            Self::default()
        }
    }
}

impl<'a, T: ModelObject> Iterator for ModelVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_ptr < self.end_ptr {
            let cur_ptr = self.cur_ptr;
            let cur_ref = unsafe { &*cur_ptr };
            self.cur_ptr = unsafe { cur_ptr.add(1) };
            return Some(cur_ref);
        }
        None
    }
}

impl Model {
    pub fn new() -> Self {
        let mut ret = Self::default();

        for native_type in NativeType::iter() {
            ret.add(CGenType::Native(native_type)).unwrap();
        }

        ret
    }

    pub fn size<T: ModelObject>(&self) -> usize {
        match self.get_container::<T>() {
            Some(e) => e.size(),
            None => 0,
        }
    }

    pub fn add<T: ModelObject>(&mut self, value: T) -> Result<ModelObjectId> {
        let key = value.key();
        if self.key_map.contains_key(&key) {
            return Err(anyhow!("Object not unique"));
        }
        let type_index = self.get_or_create_container::<T>();
        let value_ptr = &value as *const T as *const u8;
        let object_index = self.get_container_by_index_mut(type_index).add(value_ptr);
        forget(value);
        let object_id = ModelObjectId::new(
            u32::try_from(type_index).unwrap(),
            u32::try_from(object_index).unwrap(),
        );
        self.key_map.insert(key, object_id);

        Ok(object_id)
    }

    pub fn object_iter<T: ModelObject>(&self) -> ModelVecIter<'_, T> {
        let container = self.get_container::<T>();
        ModelVecIter::new(container)
    }

    pub fn get_object_id<T: ModelObject>(&self, key: ModelKey) -> Option<ModelObjectId> {
        let container_index = self.get_container_index::<T>()?;
        let id = self.key_map.get(&key).copied()?;
        assert!(id.type_index as usize == container_index);
        Some(id)
    }

    pub fn get_from_key<T: ModelObject>(&self, key: ModelKey) -> Option<&T> {
        self.get_object_id::<T>(key)
            .and_then(|x| self.get_from_objectid(x))
    }

    pub fn get_from_objectid<T: ModelObject>(&self, id: ModelObjectId) -> Option<&T> {
        let container = self.get_container::<T>()?;
        let ptr = container.get_object_ref(id.object_index as usize) as *const T;
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
        let type_index = *type_index;
        type_index
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

#[derive(Debug, Default)]
pub struct ModelContainer<T> {
    objects: HashMap<String, T>,
}

impl<T: Default> ModelContainer<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: String, entry: T) -> anyhow::Result<()> {
        if self.objects.contains_key(&id) {
            return Err(anyhow!("Object '{}' already inserted.", id));
        }
        self.objects.insert(id, entry);

        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Result<&T> {
        match self.objects.get(id) {
            Some(o) => Ok(o),
            None => Err(anyhow!("Unknown object '{}'", id)),
        }
    }

    pub fn try_get(&self, id: &str) -> Option<&T> {
        self.objects.get(id)
    }

    pub fn iter(&self) -> std::collections::hash_map::Values<'_, String, T> {
        self.objects.values()
    }
}

#[derive(Debug, Clone, Copy, EnumString, EnumIter, AsStaticStr)]
pub enum NativeType {
    Float1,
    Float2,
    Float3,
    Float4,
    Float4x4,
}

// pub type CGenTypeHandle = ModelHandle<CGenType>;

#[derive(Debug, Clone)]
pub struct StructMember {
    pub name: String,
    pub object_id: ModelObjectId,
    // pub type_handle: CGenTypeHandle,
    pub array_len: Option<u32>,
}

impl StructMember {
    pub fn new(name: &str, object_id: ModelObjectId, array_len: Option<u32>) -> Self {
        StructMember {
            name: name.to_owned(),
            object_id,
            array_len,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CGenType {
    Native(NativeType),
    Struct(StructType),
}

impl CGenType {
    pub fn struct_type(&self) -> &StructType {
        match self {
            CGenType::Struct(e) => e,
            _ => panic!("Invalid access"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub name: String,
    pub members: Vec<StructMember>,
}

impl StructType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }
}

impl ModelObject for CGenType {
    fn typename() -> &'static str {
        "CGenType"
    }
    fn name(&self) -> &str {
        match self {
            CGenType::Native(e) => e.as_static(),
            CGenType::Struct(e) => e.name.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumString, AsRefStr)]
pub enum TextureFormat {
    R8,
    R8G8B8A8,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDef {
    pub object_id: ModelObjectId,
}

#[derive(Debug)]
pub struct SamplerDescriptorDef;

#[derive(Debug, EnumString)]
pub enum DescriptorType {
    Sampler,
    ConstantBuffer,
    StructuredBuffer,
    RWStructuredBuffer,
    ByteAddressBuffer,
    RWByteAddressBuffer,
    Texture2D,
    RWTexture2D,
}

#[derive(Debug, Clone, Copy)]
pub struct ConstantBufferDef {
    pub object_id: ModelObjectId,
}

#[derive(Debug, Clone, Copy)]
pub struct StructuredBufferDef {
    pub object_id: ModelObjectId,
}

#[derive(Debug, Clone)]
pub enum DescriptorDef {
    // Sampler
    Sampler,
    // Buffers
    ConstantBuffer(ConstantBufferDef),
    StructuredBuffer(StructuredBufferDef),
    RWStructuredBuffer(StructuredBufferDef),
    ByteAddressBuffer,
    RWByteAddressBuffer,
    // Textures
    Texture2D(TextureDef),
    RWTexture2D(TextureDef),
    Texture3D(TextureDef),
    RWTexture3D(TextureDef),
    Texture2DArray(TextureDef),
    RWTexture2DArray(TextureDef),
    TextureCube(TextureDef),
    TextureCubeArray(TextureDef),
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub name: String,
    pub array_len: Option<u32>,
    pub def: DescriptorDef,
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorSet {
    pub name: String,
    pub frequency: u32,
    pub descriptors: Vec<Descriptor>,
}

impl DescriptorSet {
    pub fn new(name: &str, frequency: u32) -> Self {
        DescriptorSet {
            name: name.to_owned(),
            frequency,
            descriptors: Vec::new(),
        }
    }
}

impl ModelObject for DescriptorSet {
    fn typename() -> &'static str {
        "DescriptorSet"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct PushConstant {
    pub name: String,
    pub object_id: ModelObjectId,
}

impl PushConstant {
    pub fn new(name: &str, object_id: ModelObjectId) -> Self {
        PushConstant {
            name: name.to_owned(),
            object_id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PipelineLayoutContent {
    DescriptorSet(ModelObjectId),
    Pushconstant(ModelObjectId),
}

#[derive(Debug, Default, Clone)]
pub struct PipelineLayout {
    pub name: String,
    pub members: Vec<(String, PipelineLayoutContent)>,
}

impl PipelineLayout {
    pub fn new(name: &str) -> PipelineLayout {
        PipelineLayout {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }

    pub fn descriptor_sets(&self) -> impl Iterator<Item = ModelObjectId> + '_ {
        let x = self.members.iter().filter_map(|m| match m.1 {
            PipelineLayoutContent::DescriptorSet(ds) => Some(ds),
            PipelineLayoutContent::Pushconstant(_) => None,
        });
        x
    }
}

impl ModelObject for PipelineLayout {
    fn typename() -> &'static str {
        "PipelineLayout"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
