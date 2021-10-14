use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::Resource;

/// An asset registered in the `AssetRegistry`
pub struct RegisteredAsset(RefCell<Box<dyn Any + Send + Sync>>);

impl RegisteredAsset {
    pub(crate) fn new(asset: Box<dyn Any + Send + Sync>) -> Self {
        Self(RefCell::new(asset))
    }

    pub(crate) fn is<T>(&self) -> bool
    where
        T: Any + Resource,
    {
        <dyn Any>::is::<T>(self.0.borrow().as_ref())
    }

    pub(crate) fn borrow<T>(&self) -> AssetRef<'_, T>
    where
        T: Any + Resource,
    {
        AssetRef::new(self.0.borrow())
    }

    pub(crate) fn borrow_mut<T>(&self) -> AssetRefMut<'_, T>
    where
        T: Any + Resource,
    {
        AssetRefMut::new(self.0.borrow_mut())
    }
}

/// Borrowed reference to an asset registered in the `AssetRegistry`
pub struct AssetRef<'b, T>
where
    T: Any + Resource,
{
    reference: Ref<'b, Box<dyn Any + Send + Sync>>,
    phantom: PhantomData<T>,
}

impl<'b, T> AssetRef<'b, T>
where
    T: Any + Resource,
{
    fn new(reference: Ref<'b, Box<dyn Any + Send + Sync>>) -> Self {
        Self {
            reference,
            phantom: PhantomData,
        }
    }
}

impl<'b, T> Deref for AssetRef<'b, T>
where
    T: Any + Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reference.downcast_ref::<T>().unwrap()
    }
}

/// Mutably borrowed reference to an asset registered in the `AssetRegistry`
pub struct AssetRefMut<'b, T>
where
    T: Any + Resource,
{
    reference_mut: RefMut<'b, Box<dyn Any + Send + Sync>>,
    phantom: PhantomData<T>,
}

impl<'b, T> AssetRefMut<'b, T>
where
    T: Any + Resource,
{
    fn new(reference_mut: RefMut<'b, Box<dyn Any + Send + Sync>>) -> Self {
        Self {
            reference_mut,
            phantom: PhantomData,
        }
    }
}

impl<'b, T> Deref for AssetRefMut<'b, T>
where
    T: Any + Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reference_mut.downcast_ref::<T>().unwrap()
    }
}

impl<'b, T> DerefMut for AssetRefMut<'b, T>
where
    T: Any + Resource,
{
    fn deref_mut(&mut self) -> &mut T {
        self.reference_mut.downcast_mut::<T>().unwrap()
    }
}
