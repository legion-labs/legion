use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use lgn_utils::HashMap;

#[derive(Hash, PartialEq, Eq)]
struct ResourceId {
    type_id: TypeId,
}

impl ResourceId {
    fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }
}

pub trait RenderResource: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send + Sync> RenderResource for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct RenderResourcesMap(HashMap<ResourceId, AtomicRefCell<Box<dyn RenderResource>>>);

pub struct RenderResourcesBuilder {
    resource_map: RenderResourcesMap,
}

impl RenderResourcesBuilder {
    pub fn new() -> Self {
        Self {
            resource_map: RenderResourcesMap::default(),
        }
    }

    #[must_use]
    pub fn insert<T: RenderResource>(mut self, resource: T) -> Self {
        let resource_id = ResourceId::new::<T>();
        assert!(!self.resource_map.0.contains_key(&resource_id));
        self.resource_map
            .0
            .insert(resource_id, AtomicRefCell::new(Box::new(resource)));
        self
    }

    pub fn finalize(self) -> RenderResources {
        RenderResources::new(self.resource_map)
    }
}

struct Inner {
    resource_map: RenderResourcesMap,
}

#[derive(Clone)]
pub struct RenderResources {
    inner: Arc<Inner>,
}

impl RenderResources {
    pub fn new(resource_map: RenderResourcesMap) -> Self {
        Self {
            inner: Arc::new(Inner { resource_map }),
        }
    }

    pub fn get<T: 'static>(&self) -> ResourceHandle<'_, T> {
        let resource_id = ResourceId::new::<T>();
        let atomic_ref = self
            .inner
            .resource_map
            .0
            .get(&resource_id)
            .unwrap()
            .borrow();
        let atomic_ref = AtomicRef::map(atomic_ref, std::convert::AsRef::as_ref);
        ResourceHandle {
            atomic_ref,
            phantom: PhantomData,
        }
    }

    pub fn get_mut<T: 'static>(&self) -> ResourceHandleMut<'_, T> {
        let resource_id = ResourceId::new::<T>();
        let atomic_ref = self
            .inner
            .resource_map
            .0
            .get(&resource_id)
            .unwrap()
            .borrow_mut();
        let atomic_ref = AtomicRefMut::map(atomic_ref, std::convert::AsMut::as_mut);
        ResourceHandleMut {
            atomic_ref_mut: atomic_ref,
            phantom: PhantomData,
        }
    }
}

pub struct ResourceHandle<'a, T> {
    atomic_ref: AtomicRef<'a, dyn RenderResource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'static> Deref for ResourceHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.atomic_ref
            .deref()
            .as_any()
            .downcast_ref::<T>()
            .unwrap()
    }
}

pub struct ResourceHandleMut<'a, T> {
    atomic_ref_mut: AtomicRefMut<'a, dyn RenderResource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'static> Deref for ResourceHandleMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.atomic_ref_mut
            .deref()
            .as_any()
            .downcast_ref::<T>()
            .unwrap()
    }
}

impl<'a, T: 'static> DerefMut for ResourceHandleMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.atomic_ref_mut
            .deref_mut()
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}
