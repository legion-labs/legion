use std::{
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
    pub fn get<'a, T: Resource>(
        &'_ self,
        registry: &'a AssetRegistry,
    ) -> Option<crate::AssetRegistryGuard<'a, T>> {
        registry.get::<T>(self.inner.type_id)
    }

    /// Create a detach clone of a `Resource`
    pub fn instantiate<T: Resource>(&self, registry: &AssetRegistry) -> Option<Box<T>> {
        let asset = registry.instantiate(self.inner.type_id)?;
        if asset.is::<T>() {
            let value = unsafe { Box::from_raw(Box::into_raw(asset).cast::<T>()) };
            return Some(value);
        }
        None
    }

    /// Replace a resource
    pub fn apply<T: Resource>(&self, value: Box<T>, registry: &AssetRegistry) {
        registry.apply(self.inner.type_id, value);
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

    /// Returns a typed Handle
    pub fn typed<T: Resource>(self) -> Handle<T> {
        Handle::<T>::from(self)
    }
}

impl AsRef<Self> for HandleUntyped {
    fn as_ref(&self) -> &Self {
        self
    }
}

//
//
//

/// Typed handle to [`Resource`] of type `T`.
pub struct Handle<T: Resource> {
    handle: HandleUntyped,
    _pd: PhantomData<fn() -> T>,
}

impl<T: Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            _pd: PhantomData,
        }
    }
}

impl<T: Resource> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle.inner.type_id == other.handle.inner.type_id
    }
}

impl<T: Resource> From<HandleUntyped> for Handle<T> {
    fn from(handle: HandleUntyped) -> Self {
        Self {
            handle,
            _pd: PhantomData,
        }
    }
}

impl<T: Resource> Handle<T> {
    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a>(
        &'_ self,
        registry: &'a AssetRegistry,
    ) -> Option<crate::AssetRegistryGuard<'a, T>> {
        registry.get::<T>(self.handle.inner.type_id)
    }

    /// Returns an editable copy of a Resource
    pub fn instantiate(&self, registry: &AssetRegistry) -> Option<Box<T>> {
        let asset = registry.instantiate(self.handle.inner.type_id)?;
        if asset.is::<T>() {
            let value = unsafe { Box::from_raw(Box::into_raw(asset).cast::<T>()) };
            return Some(value);
        }
        None
    }

    /// Apply the change to a Resource
    pub fn apply(&self, value: Box<T>, registry: &AssetRegistry) {
        registry.apply(self.handle.inner.type_id, value);
    }

    /// Returns `ResourceId` associated with this handle.
    pub fn id(&self) -> ResourceTypeAndId {
        self.handle.inner.type_id
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.handle.inner.type_id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.handle.inner.type_id)
    }
}

impl<T: Resource> AsRef<HandleUntyped> for Handle<T> {
    fn as_ref(&self) -> &HandleUntyped {
        &self.handle
    }
}
