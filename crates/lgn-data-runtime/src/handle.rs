use crate::{AssetRegistry, Resource, ResourceTypeAndId};
use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

slotmap::new_key_type! {
    /// Generational handle to a Resource inside a ResourceRegistry
    pub struct AssetRegistryHandleKey;
}

/// Type-less version of [`Handle`].
pub struct HandleUntyped {
    key: AssetRegistryHandleKey,
    id: Arc<ResourceTypeAndId>,
    registry: Arc<AssetRegistry>,
}

impl Drop for HandleUntyped {
    fn drop(&mut self) {
        // If the refcount is 2, this handle is the last
        // Notify asset_registry of potential cleanup.
        if Arc::strong_count(&self.id) == 2 {
            self.registry.mark_for_cleanup(self.key);
        }
    }
}

impl Clone for HandleUntyped {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            id: self.id.clone(),
            registry: self.registry.clone(),
        }
    }
}

impl Debug for HandleUntyped {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", &self.id, self.key)
    }
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl HandleUntyped {
    pub(crate) fn new_handle(
        key: AssetRegistryHandleKey,
        id: Arc<ResourceTypeAndId>,
        registry: Arc<AssetRegistry>,
    ) -> Self {
        Self { key, id, registry }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<T: Resource>(&self) -> Option<crate::AssetRegistryGuard<'_, T>> {
        self.registry.get::<T>(self.key)
    }

    /// Retrieve the Slotmap key of a Handle
    pub fn key(&self) -> AssetRegistryHandleKey {
        self.key
    }

    /// Retrieve the `ResourceTypeAndId` of a Handle.
    pub fn id(&self) -> ResourceTypeAndId {
        *self.id
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

impl<T: Resource> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
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

impl<T: Resource> AsRef<HandleUntyped> for Handle<T> {
    fn as_ref(&self) -> &HandleUntyped {
        &self.handle
    }
}

impl<T: Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            _pd: PhantomData,
        }
    }
}

impl<T: Resource> Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.handle.key)
    }
}

impl<T: Resource> Handle<T> {
    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get(&self) -> Option<crate::AssetRegistryGuard<'_, T>> {
        self.handle.registry.get(self.handle.key)
    }

    pub(crate) fn key(&self) -> AssetRegistryHandleKey {
        self.handle.key()
    }

    /// Retrieve the `ResourceTypeAndId` of a Handle.
    pub fn id(&self) -> ResourceTypeAndId {
        self.handle.id()
    }
}

/// Editable Handle to a Resource copy that can be committed
pub struct EditHandleUntyped {
    pub(crate) handle: HandleUntyped,
    pub(crate) asset: Box<dyn Resource>,
}

impl EditHandleUntyped {
    /// Return a new Editable copy of a Resource that can be commited
    pub fn new(handle: HandleUntyped, asset: Box<dyn Resource>) -> Self {
        Self { handle, asset }
    }
}

impl Deref for EditHandleUntyped {
    type Target = dyn Resource;
    fn deref(&self) -> &Self::Target {
        self.asset.as_ref()
    }
}

impl DerefMut for EditHandleUntyped {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.asset.as_mut()
    }
}

/// Editable Handle to a Resource copy that can be committed
pub struct EditHandle<T: Resource> {
    pub(crate) handle: Handle<T>,
    pub(crate) asset: Box<T>,
}

impl<T: Resource> EditHandle<T> {
    /// Return a new Editable copy of a Resource that can be commited
    pub fn new(handle: Handle<T>, asset: Box<T>) -> Self {
        Self { handle, asset }
    }
}

impl<T: Resource> Deref for EditHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.asset.as_ref()
    }
}

impl<T: Resource> DerefMut for EditHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.asset.as_mut()
    }
}
