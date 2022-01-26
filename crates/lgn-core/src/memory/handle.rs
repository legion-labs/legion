use std::ops::{Deref, DerefMut};

/// Wrapper that checks that the ownership of the object wrapped was
/// given away at a time of drop
pub struct Handle<T> {
    inner: Option<T>,
}

impl<T> Handle<T> {
    /// Create handle taking ownership of an object
    pub fn new(data: T) -> Self {
        Self { inner: Some(data) }
    }

    /// Check if this handle owns anything
    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// Take the wrapped object out of the Handle
    /// leaving None behind
    pub fn take(&mut self) -> T {
        match self.inner.take() {
            Some(e) => e,
            None => unreachable!(),
        }
    }

    /// Creates a new handle leaving self empty
    pub fn transfer(&mut self) -> Self {
        Self {
            inner: Some(self.take()),
        }
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        match &self.inner {
            Some(_) => unreachable!("This handle (of type {}) should have been released. It should not have the ownership of the internal resource.", std::any::type_name::<T>()),
            None => (),
        }
    }
}
