use std::ops::{Deref, DerefMut};

pub struct RenderHandle<T> {
    inner: Option<T>,
}

impl<T> RenderHandle<T> {
    pub fn new(data: T) -> Self {
        Self { inner: Some(data) }
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn take(&mut self) -> T {
        match self.inner.take() {
            Some(e) => e,
            None => unreachable!(),
        }
    }

    pub fn as_ref(&self) -> &T {
        match &self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }

    pub fn transfer(&mut self) -> Self {
        Self {
            inner: Some(self.take()),
        }
    }
}

impl<T> Deref for RenderHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> DerefMut for RenderHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> Drop for RenderHandle<T> {
    fn drop(&mut self) {
        match &self.inner {
            Some(_) => unreachable!("This handle (of type {}) should have been released. It should not have the ownership of the internal resource.", std::any::type_name::<T>()),
            None => (),
        }
    }
}
