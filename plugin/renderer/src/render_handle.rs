use std::ops::{Deref, DerefMut};

use crate::resources::GpuSafeRotate;

pub struct RendererHandle<T> {
    inner: Option<T>,
}

impl<T> RendererHandle<T> {
    pub fn new(data: T) -> Self {
        Self { inner: Some(data) }
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn peek(&mut self) -> T {
        match self.inner.take() {
            Some(e) => e,
            None => unreachable!(),
        }
    }

    pub fn take(&mut self) -> Self {
        Self {
            inner: self.inner.take(),
        }
    }
}

impl<T> Deref for RendererHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> DerefMut for RendererHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            Some(e) => e,
            None => unreachable!(),
        }
    }
}

impl<T> Drop for RendererHandle<T> {
    fn drop(&mut self) {
        match &self.inner {
            Some(_) => unreachable!("This handle should have been released. It should not have the ownership of the internal resource."),
            None => (),
        }
    }
}

impl<T: GpuSafeRotate> GpuSafeRotate for RendererHandle<T> {
    fn rotate(&mut self) {
        match &mut self.inner {
            Some(e) => e.rotate(),
            None => unreachable!(),
        }
    }
}
