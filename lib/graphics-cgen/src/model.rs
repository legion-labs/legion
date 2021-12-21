#![allow(unsafe_code)]

use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::forget;
use std::ptr::{null, NonNull};

use anyhow::{anyhow, Result};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ModelKey(u64);

impl ModelKey {
    fn new(s: &str) -> Self {
        let mut hasher = DefaultHasher::default();
        s.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug)]
struct ModelVec {
    id: u32,
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: NonNull<u8>,
    drop_fn: unsafe fn(*mut u8),
}

impl ModelVec {
    fn new(id: u32, item_layout: Layout, drop_fn: unsafe fn(*mut u8)) -> Self {
        Self {
            id,
            item_layout,
            capacity: 0,
            len: 0,
            data: NonNull::dangling(),
            drop_fn,
        }
    }

    fn id(&self) -> u32 {
        self.id
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

pub trait ModelObject: 'static + Clone + Sized + Hash + PartialEq {
    fn typename() -> &'static str;
    fn name(&self) -> &str;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ModelObjectRef<T>
where
    T: ModelObject,
{
    object_id: ModelObjectId,
    _phantom: PhantomData<*const T>,
}

impl<T> ModelObjectRef<T>
where
    T: ModelObject,
{
    pub fn new(object_id: ModelObjectId) -> Self {
        Self {
            object_id,
            _phantom: PhantomData,
        }
    }

    pub fn get<'model>(&self, model: &'model Model) -> &'model T {
        model.get_from_objectid(self.object_id).unwrap()
    }

    pub fn id(&self) -> u32 {
        self.object_id.object_index
    }
}

impl<T> Copy for ModelObjectRef<T> where T: ModelObject {}

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

pub struct ModelRefIter<'a, T: ModelObject> {
    type_idx: u32,
    cur_idx: u32,
    last_idx: u32,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ModelObject> Default for ModelRefIter<'a, T> {
    fn default() -> Self {
        Self {
            type_idx: u32::MAX,
            cur_idx: 0,
            last_idx: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ModelObject> ModelRefIter<'a, T> {
    fn new(model_vec: Option<&'a ModelVec>) -> Self {
        if let Some(model_vec) = model_vec {
            Self {
                type_idx: model_vec.id(),
                cur_idx: 0,
                last_idx: u32::try_from(model_vec.size()).unwrap(),
                _marker: PhantomData::default(),
            }
        } else {
            Self::default()
        }
    }
}

impl<'a, T: ModelObject> Iterator for ModelRefIter<'a, T> {
    type Item = ModelObjectRef<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx < self.last_idx {
            let cur_idx = self.cur_idx;
            self.cur_idx += 1;
            return Some(ModelObjectRef::<T>::new(ModelObjectId::new(
                self.type_idx,
                cur_idx,
            )));
        }
        None
    }
}

impl Model {
    pub fn new() -> Self {
        let mut ret = Self::default();

        for native_type in NativeType::iter() {
            ret.add(native_type.into(), CGenType::Native(native_type))
                .unwrap();
        }

        ret
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
    pub fn add<T: ModelObject>(&mut self, key: &str, value: T) -> Result<ModelObjectRef<T>> {
        let key = ModelKey::new(key);
        if self.key_map.contains_key(&key) {
            return Err(anyhow!("Object not unique"));
        }
        let type_index = self.get_or_create_container::<T>();
        let value_ptr = (&value as *const T).cast::<u8>();
        let object_index = self.get_container_by_index_mut(type_index).add(value_ptr);
        forget(value);
        let object_id = ModelObjectId::new(
            u32::try_from(type_index).unwrap(),
            u32::try_from(object_index).unwrap(),
        );
        self.key_map.insert(key, object_id);

        Ok(ModelObjectRef::new(object_id))
    }

    pub fn object_iter<T: ModelObject>(&self) -> ModelVecIter<'_, T> {
        let container = self.get_container::<T>();
        ModelVecIter::new(container)
    }

    pub fn ref_iter<T: ModelObject>(&self) -> ModelRefIter<'_, T> {
        let container = self.get_container::<T>();
        ModelRefIter::new(container)
    }

    pub fn get_object_ref<T: ModelObject>(&self, key: &str) -> Option<ModelObjectRef<T>> {
        let container_index = self.get_container_index::<T>()?;
        let key = ModelKey::new(key);
        let id = self.key_map.get(&key).copied()?;
        assert!(id.type_index as usize == container_index);
        Some(ModelObjectRef::new(id))
    }

    fn get_from_objectid<T: ModelObject>(&self, id: ModelObjectId) -> Option<&T> {
        let container = self.get_container::<T>()?;
        let ptr = container
            .get_object_ref(id.object_index as usize)
            .cast::<T>();
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
                .push(ModelVec::new(u32::try_from(index).unwrap(), Layout::new::<T>(), drop_ptr::<T>));
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, EnumString, EnumIter, IntoStaticStr)]
pub enum NativeType {
    Float1,
    Float2,
    Float3,
    Float4,
    Float4x4,
}

// pub type CGenTypeHandle = ModelHandle<CGenType>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructMember {
    pub name: String,
    pub ty_ref: CGenTypeRef,
    pub array_len: Option<u32>,
}

impl StructMember {
    pub fn new(name: &str, ty_ref: CGenTypeRef, array_len: Option<u32>) -> Self {
        Self {
            name: name.to_owned(),
            ty_ref,
            array_len,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CGenType {
    Native(NativeType),
    Struct(StructType),
}

impl CGenType {
    pub fn struct_type(&self) -> &StructType {
        match self {
            CGenType::Struct(e) => e,
            CGenType::Native(_) => panic!("Invalid access"),
        }
    }
}

pub type CGenTypeRef = ModelObjectRef<CGenType>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
        "CgenType"
    }
    fn name(&self) -> &str {
        match self {
            CGenType::Native(e) => e.into(),
            CGenType::Struct(e) => e.name.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumString, AsRefStr)]
pub enum TextureFormat {
    R8,
    R8G8B8A8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextureDef {
    pub ty_ref: CGenTypeRef,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConstantBufferDef {
    pub ty_ref: CGenTypeRef,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructuredBufferDef {
    pub ty_ref: CGenTypeRef,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Descriptor {
    pub name: String,
    pub array_len: Option<u32>,
    pub def: DescriptorDef,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DescriptorSet {
    pub name: String,
    pub frequency: u32,
    pub descriptors: Vec<Descriptor>,
}

pub type DescriptorSetRef = ModelObjectRef<DescriptorSet>;

impl DescriptorSet {
    pub fn new(name: &str, frequency: u32) -> Self {
        assert!((frequency as usize) < MAX_DESCRIPTOR_SET_LAYOUTS);
        Self {
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
    pub ty_ref: CGenTypeRef,
}

impl PushConstant {
    pub fn new(name: &str, ty_ref: CGenTypeRef) -> Self {
        Self {
            name: name.to_owned(),
            ty_ref,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PipelineLayoutContent {
    DescriptorSet(DescriptorSetRef),
    Pushconstant(CGenTypeRef),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineLayout {
    pub name: String,
    pub members: Vec<(String, PipelineLayoutContent)>,
}

pub type PipelineLayoutRef = ModelObjectRef<PipelineLayout>;

impl PipelineLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }

    pub fn descriptor_sets(&self) -> impl Iterator<Item = DescriptorSetRef> + '_ {
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
