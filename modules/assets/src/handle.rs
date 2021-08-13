use std::{marker::PhantomData, sync::mpsc};

use crate::{Asset, AssetRegistry};

pub(crate) type HandleId = u32;

pub(crate) enum RefOp {
    AddRef(HandleId),
    RemoveRef(HandleId),
}

/// Type-less version of [`Handle`].
#[derive(Debug)]
pub struct HandleUntyped {
    pub(crate) id: HandleId,
    refcount_tx: mpsc::Sender<RefOp>,
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
    pub(crate) fn create(id: HandleId, refcount_tx: mpsc::Sender<RefOp>) -> Self {
        Self { id, refcount_tx }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a, T: Asset>(&'_ self, registry: &'a AssetRegistry) -> Option<&'a T> {
        registry.get::<T>(self.id)
    }

    /// Returns true if [`Asset`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}

/// Typed handle to [`Asset`] of type `T`.
pub struct Handle<'a, T: Asset> {
    id: HandleId,
    refcount_tx: mpsc::Sender<RefOp>,
    _pd: PhantomData<&'a T>,
}

impl<T: Asset> Drop for Handle<'_, T> {
    fn drop(&mut self) {
        self.refcount_tx.send(RefOp::RemoveRef(self.id)).unwrap();
    }
}

impl<T: Asset> Clone for Handle<'_, T> {
    fn clone(&self) -> Self {
        self.refcount_tx.send(RefOp::AddRef(self.id)).unwrap();
        Self {
            id: self.id,
            refcount_tx: self.refcount_tx.clone(),
            _pd: PhantomData {},
        }
    }
}

impl<T: Asset> PartialEq for Handle<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> Handle<'_, T> {
    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a>(&'_ self, registry: &'a AssetRegistry) -> Option<&'a T> {
        registry.get::<T>(self.id)
    }

    /// Returns true if [`Asset`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}
