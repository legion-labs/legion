use std::marker::PhantomData;

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
    pub fn get<'a, T: Asset>(&'_ self, registry: &'a AssetRegistry) -> Option<&'a T> {
        registry.get_untyped::<T>(self)
    }

    /// Returns true if [`Asset`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}

/// Typed handle to [`Asset`] of type `T`.
pub struct Handle<T: Asset> {
    pub(crate) id: HandleId,
    refcount_tx: crossbeam_channel::Sender<RefOp>,
    _pd: PhantomData<fn() -> T>,
}

impl<T: Asset> Drop for Handle<T> {
    fn drop(&mut self) {
        self.refcount_tx.send(RefOp::RemoveRef(self.id)).unwrap();
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        self.refcount_tx.send(RefOp::AddRef(self.id)).unwrap();
        Self {
            id: self.id,
            refcount_tx: self.refcount_tx.clone(),
            _pd: PhantomData {},
        }
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> From<HandleUntyped> for Handle<T> {
    fn from(handle: HandleUntyped) -> Self {
        handle
            .refcount_tx
            .send(RefOp::AddRef(handle.id))
            .expect("asset loader to exist");
        Self::create(handle.id, handle.refcount_tx.clone())
    }
}

impl<T: Asset> Handle<T> {
    pub(crate) fn create(id: HandleId, refcount_tx: crossbeam_channel::Sender<RefOp>) -> Self {
        Self {
            id,
            refcount_tx,
            _pd: PhantomData {},
        }
    }

    /// Retrieve a reference asset `T` from [`AssetRegistry`].
    pub fn get<'a>(&'_ self, registry: &'a AssetRegistry) -> Option<&'a T> {
        registry.get::<T>(self)
    }

    /// Returns true if [`Asset`] load failed.
    pub fn is_err(&self, registry: &AssetRegistry) -> bool {
        registry.is_err(self.id)
    }
}
