use crate::{AssetRegistry, Resource, ResourceTypeAndId};
use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard},
};

slotmap::new_key_type! {
    /// Generational handle to a Resource inside a ResourceRegistry
    pub struct AssetRegistryHandleKey;
}

pub(crate) struct HandleEntry {
    pub(crate) id: ResourceTypeAndId,
    pub(crate) asset: RwLock<Box<dyn Resource>>,
    registry: std::sync::Weak<AssetRegistry>,
}

impl HandleEntry {
    pub(crate) fn new(
        id: ResourceTypeAndId,
        asset: Box<dyn Resource>,
        registry: std::sync::Weak<AssetRegistry>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            asset: RwLock::new(asset),
            registry,
        })
    }
}

/// Type-less version of [`Handle`].
pub struct HandleUntyped {
    key: AssetRegistryHandleKey,
    entry: Arc<HandleEntry>,
}

impl Drop for HandleUntyped {
    fn drop(&mut self) {
        // If the refcount is 2, this handle is the last
        // Notify asset_registry of potential cleanup.
        if Arc::strong_count(&self.entry) == 2 {
            if let Some(registry) = self.entry.registry.upgrade() {
                registry.mark_for_cleanup(self.key);
            }
        }
    }
}

impl Clone for HandleUntyped {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            entry: self.entry.clone(),
        }
    }
}

impl Debug for HandleUntyped {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", &self.entry.id, self.key)
    }
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
}

/// Return a Guarded Ref to a Asset
pub struct HandleGuard<'a, T: ?Sized + 'a> {
    _guard: RwLockReadGuard<'a, Box<dyn Resource>>,
    ptr: *const T,
}

impl<'a, T: ?Sized + 'a> std::ops::Deref for HandleGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl HandleUntyped {
    pub(crate) fn new(key: AssetRegistryHandleKey, entry: Arc<HandleEntry>) -> Self {
        Self { key, entry }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get_untyped(&self) -> Option<crate::HandleGuard<'_, dyn Resource>> {
        let guard = self.entry.asset.read().unwrap();
        let ptr = guard.as_ref() as *const dyn Resource;
        Some(HandleGuard { _guard: guard, ptr })
    }

    /// Retrieve the Slotmap key of a Handle
    pub fn key(&self) -> AssetRegistryHandleKey {
        self.key
    }

    /// Retrieve the `ResourceTypeAndId` of a Handle.
    pub fn id(&self) -> ResourceTypeAndId {
        self.entry.id
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
    pub fn get(&self) -> Option<crate::HandleGuard<'_, T>> {
        let guard = self.handle.entry.asset.read().unwrap();
        if let Some(ptr) = guard.as_ref().downcast_ref::<T>().map(|c| c as *const T) {
            return Some(HandleGuard { _guard: guard, ptr });
        }
        None
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
