use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{AssetRegistry, Resource, ResourceTypeAndId};

//
//
//

/// Arc<Inner> is responsible for sending a 'unload' message when last reference
/// is dropped.
#[derive(Debug)]
struct Inner {
    type_id: ResourceTypeAndId,
    unload_tx: Option<crossbeam_channel::Sender<ResourceTypeAndId>>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        let _ = self.unload_tx.as_ref().map(|tx| tx.send(self.type_id));
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
    /// Attempts to upgrade the non-owning reference to an owning
    /// `HandleUntyped`.
    ///
    /// Returns [`None`] if the inner value has since been dropped.
    pub fn upgrade(&self) -> Option<HandleUntyped> {
        self.inner.upgrade().map(HandleUntyped::from_inner)
    }

    /// Gets the number of strong ([`HandleUntyped`] and [`Handle`]) pointers
    /// pointing to a Resource.
    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }
}

//
//
//

/// Type-less version of [`Handle`].
#[derive(Debug, Clone)]
pub struct HandleUntyped {
    inner: Arc<Inner>,
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.inner.type_id == other.inner.type_id
    }
}

impl HandleUntyped {
    pub(crate) fn new_handle(
        type_id: ResourceTypeAndId,
        handle_drop_tx: crossbeam_channel::Sender<ResourceTypeAndId>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                type_id,
                unload_tx: Some(handle_drop_tx),
            }),
        }
    }

    fn from_inner(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    pub(crate) fn forget(self) {
        let mut inner = Arc::try_unwrap(self.inner).unwrap();
        let _discard = inner.unload_tx.take();
    }

    pub(crate) fn downgrade(this: &Self) -> ReferenceUntyped {
        ReferenceUntyped {
            inner: Arc::downgrade(&this.inner),
        }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a, T: Any + Resource>(
        &'_ self,
        registry: &'a AssetRegistry,
    ) -> Option<crate::AssetRegistryGuard<'a, T>> {
        registry.get::<T>(self.inner.type_id)
    }

    /// Returns `ResourceId` associated with this handle.
    pub fn id(&self) -> ResourceTypeAndId {
        self.inner.type_id
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.inner.type_id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.inner.type_id)
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

impl<T: Any + Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _pd: PhantomData,
        }
    }
}

impl<T: Any + Resource> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.type_id == other.inner.type_id
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
    pub fn get<'a>(
        &'_ self,
        registry: &'a AssetRegistry,
    ) -> Option<crate::AssetRegistryGuard<'a, T>> {
        registry.get::<T>(self.inner.type_id)
    }

    /// Returns `ResourceId` associated with this handle.
    pub fn id(&self) -> ResourceTypeAndId {
        self.inner.type_id
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.inner.type_id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.inner.type_id)
    }
}
