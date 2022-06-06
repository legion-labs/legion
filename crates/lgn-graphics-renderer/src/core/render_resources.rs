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

pub trait RenderResource: 'static + Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> RenderResource for T
where
    T: Any + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// #[derive(Default)]
// pub struct RenderResourcesMap(HashMap<ResourceId, AtomicRefCell<Box<dyn RenderResource>>>);

pub struct RenderResourcesBuilder {
    resource_map: HashMap<ResourceId, Box<dyn RenderResource>>,
}

impl RenderResourcesBuilder {
    pub fn new() -> Self {
        Self {
            resource_map: HashMap::default(),
        }
    }

    #[must_use]
    pub fn insert<T: RenderResource>(mut self, resource: T) -> Self {
        let resource_id = ResourceId::new::<T>();
        assert!(!self.resource_map.contains_key(&resource_id));
        self.resource_map.insert(resource_id, Box::new(resource));
        self
    }

    pub fn finalize(mut self) -> RenderResources {
        let mut render_resources_map = HashMap::new();
        let mut render_resources = Vec::new();

        for (resource_id, resource) in self.resource_map.drain() {
            render_resources_map.insert(resource_id, render_resources.len());
            render_resources.push(AtomicRefCell::new(resource));
        }

        RenderResources {
            inner: Arc::new(Inner {
                render_resources_map,
                render_resources,
            }),
        }
    }
}

struct Inner {
    render_resources_map: HashMap<ResourceId, usize>,
    render_resources: Vec<AtomicRefCell<Box<dyn RenderResource>>>,
}

#[derive(Clone)]
pub struct RenderResources {
    inner: Arc<Inner>,
}

impl RenderResources {
    pub fn get<T: 'static>(&self) -> ResourceHandle<'_, T> {
        self.try_get().unwrap()
    }

    pub fn try_get<T: 'static>(&self) -> Option<ResourceHandle<'_, T>> {
        let resource_id = ResourceId::new::<T>();
        self.try_get_cell(&resource_id).map(|x| ResourceHandle {
            atomic_ref: AtomicRef::map(x.borrow(), std::convert::AsRef::as_ref),
            phantom: PhantomData,
        })
    }

    pub fn get_mut<T: 'static>(&self) -> ResourceHandleMut<'_, T> {
        self.try_mut().unwrap()
    }

    pub fn try_mut<T: 'static>(&self) -> Option<ResourceHandleMut<'_, T>> {
        let resource_id = ResourceId::new::<T>();
        self.try_get_cell(&resource_id).map(|x| ResourceHandleMut {
            atomic_ref_mut: AtomicRefMut::map(x.borrow_mut(), std::convert::AsMut::as_mut),
            phantom: PhantomData,
        })
    }

    fn try_get_cell(
        &self,
        resource_id: &ResourceId,
    ) -> Option<&AtomicRefCell<Box<dyn RenderResource>>> {
        let index = self.inner.render_resources_map.get(resource_id);
        index.map(|index| &self.inner.render_resources[*index])
    }
}

// SAFETY: RenderResources is an immutable structure behind an Arc
#[allow(unsafe_code)]
unsafe impl Send for RenderResources {}

// SAFETY: RenderResources is an immutable structure behind an Arc
#[allow(unsafe_code)]
unsafe impl Sync for RenderResources {}

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
