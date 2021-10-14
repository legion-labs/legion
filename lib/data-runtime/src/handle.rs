use std::{any::Any, marker::PhantomData};

use crate::{AssetRef, AssetRefMut, AssetRegistry, RegisteredAsset, Resource, ResourceId};

pub(crate) type HandleId = u32;

pub(crate) enum RefOp {
    AddRef(HandleId),
    RemoveRef(HandleId),
}

/// Type-less version of [`Handle`].
#[derive(Debug)]
pub struct HandleUntyped {
    pub(crate) id: HandleId,
    refcount_tx: crossbeam_channel::Sender<RefOp>,
}

impl Drop for HandleUntyped {
    fn drop(&mut self) {
        self.refcount_tx.send(RefOp::RemoveRef(self.id)).unwrap();
    }
}

impl Clone for HandleUntyped {
    fn clone(&self) -> Self {
        self.refcount_tx.send(RefOp::AddRef(self.id)).unwrap();
        Self {
            id: self.id,
            refcount_tx: self.refcount_tx.clone(),
        }
    }
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl HandleUntyped {
    pub(crate) fn create(id: HandleId, refcount_tx: crossbeam_channel::Sender<RefOp>) -> Self {
        Self { id, refcount_tx }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a, T>(&'_ self, registry: &'a AssetRegistry) -> Option<AssetRef<'a, T>>
    where
        T: Any + Resource,
    {
        registry.get::<T>(self.id).map(RegisteredAsset::borrow)
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get_mut<'a, T>(&'_ self, registry: &'a AssetRegistry) -> Option<AssetRefMut<'a, T>>
    where
        T: Any + Resource,
    {
        registry.get::<T>(self.id).map(RegisteredAsset::borrow_mut)
    }

    /// Retrieves the asset id associated with this handle within the [`AssetRegistry`].
    pub fn get_asset_id(&self, registry: &AssetRegistry) -> Option<ResourceId> {
        registry.get_asset_id(self.id)
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}

/// Typed handle to [`Resource`] of type `T`.
pub struct Handle<T: Any + Resource> {
    pub(crate) id: HandleId,
    refcount_tx: crossbeam_channel::Sender<RefOp>,
    _pd: PhantomData<fn() -> T>,
}

impl<T: Any + Resource> Drop for Handle<T> {
    fn drop(&mut self) {
        self.refcount_tx.send(RefOp::RemoveRef(self.id)).unwrap();
    }
}

impl<T: Any + Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        self.refcount_tx.send(RefOp::AddRef(self.id)).unwrap();
        Self {
            id: self.id,
            refcount_tx: self.refcount_tx.clone(),
            _pd: PhantomData {},
        }
    }
}

impl<T: Any + Resource> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Any + Resource> From<HandleUntyped> for Handle<T> {
    fn from(handle: HandleUntyped) -> Self {
        handle
            .refcount_tx
            .send(RefOp::AddRef(handle.id))
            .expect("asset loader to exist");
        Self::create(handle.id, handle.refcount_tx.clone())
    }
}

impl<T: Any + Resource> Handle<T> {
    pub(crate) fn create(id: HandleId, refcount_tx: crossbeam_channel::Sender<RefOp>) -> Self {
        Self {
            id,
            refcount_tx,
            _pd: PhantomData {},
        }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a>(&'_ self, registry: &'a AssetRegistry) -> Option<AssetRef<'a, T>>
    where
        T: Any + Resource,
    {
        registry.get::<T>(self.id).map(RegisteredAsset::borrow)
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get_mut<'a>(&'_ self, registry: &'a AssetRegistry) -> Option<AssetRefMut<'a, T>>
    where
        T: Any + Resource,
    {
        registry.get::<T>(self.id).map(RegisteredAsset::borrow_mut)
    }

    /// Retrieves the asset id associated with this handle within the [`AssetRegistry`].
    pub fn get_asset_id(&self, registry: &AssetRegistry) -> Option<ResourceId> {
        registry.get_asset_id(self.id)
    }

    /// Returns true if [`Resource`] load is finished and has succeeded.
    pub fn is_loaded(&self, registry: &AssetRegistry) -> bool {
        registry.is_loaded(self.id)
    }

    /// Returns true if [`Resource`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}
