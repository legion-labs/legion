use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{AssetRegistry, Ref, Resource, ResourceId};

//
//
//

/// Arc<Inner> is responsible for sending a 'unload' message when last reference is dropped.
#[derive(Debug)]
struct Inner {
    id: ResourceId,
    unload_tx: crossbeam_channel::Sender<ResourceId>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.unload_tx.send(self.id).unwrap();
    }
}

//
//
//

/// Non-owning reference to a Resource.
pub struct ReferenceUntyped {
    inner: Weak<Inner>,
}

impl ReferenceUntyped {
    /// Attempts to upgrade the non-owning reference to an owning `HandleUntyped`.
    ///
    /// Returns [`None`] if the inner value has since been dropped.
    pub fn upgrade(&self) -> Option<HandleUntyped> {
        self.inner.upgrade().map(HandleUntyped::from_inner)
    }

    /// Gets the number of strong ([`HandleUntyped`] and [`Handle`]) pointers pointing to a Resource.
    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }
}

//
//
//

/// Type-less version of [`Handle`].
#[derive(Debug)]
pub struct HandleUntyped {
    inner: Arc<Inner>,
}

impl Clone for HandleUntyped {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl HandleUntyped {
    pub(crate) fn new_handle(
        id: ResourceId,
        handle_drop_tx: crossbeam_channel::Sender<ResourceId>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                id,
                unload_tx: handle_drop_tx,
            }),
        }
    }

    fn from_inner(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    pub(crate) fn downgrade(this: &Self) -> ReferenceUntyped {
        ReferenceUntyped {
            inner: Arc::downgrade(&this.inner),
        }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a, T: Any + Resource>(&'_ self, registry: &'a AssetRegistry) -> Option<Ref<'a, T>> {
        registry.get::<T>(self.inner.id)
    }

    /// Returns `ResourceId` associated with this handle.
    pub fn id(&self) -> ResourceId {
        self.inner.id
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.inner.id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.inner.id)
    }
}

//
//
//

/// Typed handle to [`Resource`] of type `T`.
pub struct Handle<T: Any + Resource> {
    inner: Arc<Inner>,
    _pd: PhantomData<fn() -> T>,
}

impl<T: Any + Resource> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<T: Any + Resource> From<HandleUntyped> for Handle<T> {
    fn from(handle: HandleUntyped) -> Self {
        Self {
            inner: handle.inner,
            _pd: PhantomData,
        }
    }
}

impl<T: Any + Resource> Handle<T> {
    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a>(&'_ self, registry: &'a AssetRegistry) -> Option<Ref<'a, T>> {
        registry.get::<T>(self.inner.id)
    }

    /// Returns `ResourceId` associated with this handle.
    pub fn id(&self) -> ResourceId {
        self.inner.id
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.inner.id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.inner.id)
    }
}
