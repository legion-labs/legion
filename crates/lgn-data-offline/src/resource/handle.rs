use std::{any::Any, marker::PhantomData};

use lgn_data_runtime::Resource;

use super::ResourceRegistry;

pub(crate) type ResourceHandleId = u32;

pub(crate) enum RefOp {
    AddRef(ResourceHandleId),
    RemoveRef(ResourceHandleId),
}

/// Type-less version of [`ResourceHandle`].
#[derive(Debug)]
pub struct ResourceHandleUntyped {
    pub(crate) id: ResourceHandleId,
    refcount_tx: crossbeam_channel::Sender<RefOp>,
}

impl AsRef<Self> for ResourceHandleUntyped {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Drop for ResourceHandleUntyped {
    fn drop(&mut self) {
        self.refcount_tx.send(RefOp::RemoveRef(self.id)).unwrap();
    }
}

impl Clone for ResourceHandleUntyped {
    fn clone(&self) -> Self {
        self.refcount_tx.send(RefOp::AddRef(self.id)).unwrap();
        Self {
            id: self.id,
            refcount_tx: self.refcount_tx.clone(),
        }
    }
}

impl PartialEq for ResourceHandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl ResourceHandleUntyped {
    pub(crate) fn create(
        id: ResourceHandleId,
        refcount_tx: crossbeam_channel::Sender<RefOp>,
    ) -> Self {
        Self { id, refcount_tx }
    }

    /// Retrieve a reference to resource of type `T` from [`ResourceRegistry`].
    pub fn get<'a, T: Any + Resource>(&'_ self, registry: &'a ResourceRegistry) -> Option<&'a T> {
        let resource = registry.get(self)?;
        resource.downcast_ref::<T>()
    }

    /// Retrieve a mutable reference to resource of type `T` from
    /// [`ResourceRegistry`].
    pub fn get_mut<'a, T: Any + Resource>(
        &'_ self,
        registry: &'a mut ResourceRegistry,
    ) -> Option<&'a mut T> {
        let resource = registry.get_mut(self)?;
        resource.downcast_mut::<T>()
    }

    /// Converts the untyped handle into a typed handle.
    pub fn typed<T: Any + Resource>(self) -> ResourceHandle<T> {
        let v = ResourceHandle::<T>::create(self.id, self.refcount_tx.clone());
        // the intent here is to not decrement the refcount as the newly returned `v`
        // will take care of it when it goes out of scope. mem::forget stops the
        // destructor of self from running.
        #[allow(clippy::mem_forget)]
        std::mem::forget(self);
        v
    }
}

/// Typed handle to [`Resource`] of type `T`.
pub struct ResourceHandle<T: Any + Resource> {
    internal: ResourceHandleUntyped,
    _pd: PhantomData<fn() -> T>,
}

impl<T: Any + Resource> AsRef<ResourceHandleUntyped> for ResourceHandle<T> {
    fn as_ref(&self) -> &ResourceHandleUntyped {
        &self.internal
    }
}

impl<T: Any + Resource> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        let cloned = self.internal.clone();
        Self {
            internal: cloned,
            _pd: PhantomData {},
        }
    }
}

impl<T: Any + Resource> PartialEq for ResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.internal.id == other.internal.id
    }
}

impl<T: Any + Resource> ResourceHandle<T> {
    pub(crate) fn create(
        id: ResourceHandleId,
        refcount_tx: crossbeam_channel::Sender<RefOp>,
    ) -> Self {
        Self {
            internal: ResourceHandleUntyped::create(id, refcount_tx),
            _pd: PhantomData {},
        }
    }

    /// Retrieve a reference to resource of type `T` from [`ResourceRegistry`].
    pub fn get<'a>(&'_ self, registry: &'a ResourceRegistry) -> Option<&'a T> {
        let resource = registry.get(&self.internal)?;
        resource.downcast_ref::<T>()
    }

    /// Retrieve a mutable reference to resource of type `T` from
    /// [`ResourceRegistry`].
    pub fn get_mut<'a>(&'_ self, registry: &'a mut ResourceRegistry) -> Option<&'a mut T> {
        let resource = registry.get_mut(&self.internal)?;
        resource.downcast_mut::<T>()
    }
}
