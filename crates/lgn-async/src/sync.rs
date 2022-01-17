//! Synchronization primitives for the async world.

use std::{
    future::Future,
    hint::unreachable_unchecked,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use tokio::sync::{Mutex, MutexGuard};

/// `LazyMutex` is an async Mutex - not unlike `tokio::sync::Mutex` - that lazy
/// initializes its value asynchronously the first time it gets locked.
///
/// Here is a trivial example usage:
///
/// ```
/// # use lgn_async::sync::LazyMutex;
/// #
/// #[tokio::main]
/// async fn main() {
///     let a = 42;
///     let mutex = LazyMutex::new(async move { a });
///     let lock = mutex.lock().await;
///
///     assert_eq!(*lock, 42);
/// }
/// ```
pub struct LazyMutex<T> {
    value: Mutex<LazyMutexValue<T>>,
}

/// An async mutex guard that holds onto a lock.
pub struct LazyMutexGuard<'a, T> {
    guard: MutexGuard<'a, LazyMutexValue<T>>,
}

impl<'a, T> Deref for LazyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.value()
    }
}

impl<'a, T> DerefMut for LazyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.value_mut()
    }
}

impl<T> LazyMutex<T> {
    /// Instanciate a new `LazyMutex` from a future that returns its future
    /// value.
    pub fn new<F>(f: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            value: Mutex::new(LazyMutexValue::Future(Box::pin(f))),
        }
    }

    /// Lock the mutex asynchronously, initializing it if it wasn't already.
    pub async fn lock(&self) -> LazyMutexGuard<'_, T> {
        let mut guard = self.value.lock().await;

        if let LazyMutexValue::Future(f) = &mut *guard {
            *guard.deref_mut() = LazyMutexValue::Value(f.await);
        }

        // Warning! A LazyMutexGuard should *always* contain a
        // LazyMutexValue::Value variant or the code has UB.
        LazyMutexGuard { guard }
    }
}

enum LazyMutexValue<T> {
    Future(Pin<Box<dyn Future<Output = T> + Send + 'static>>),
    Value(T),
}

#[allow(unsafe_code)]
impl<T> LazyMutexValue<T> {
    fn value(&self) -> &T {
        match self {
            LazyMutexValue::Value(v) => v,
            LazyMutexValue::Future(_) => unsafe {
                unreachable_unchecked();
            },
        }
    }

    fn value_mut(&mut self) -> &mut T {
        match self {
            LazyMutexValue::Value(v) => v,
            LazyMutexValue::Future(_) => unsafe {
                unreachable_unchecked();
            },
        }
    }
}
