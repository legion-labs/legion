#![allow(clippy::inline_always)]

use std::sync::Mutex;

use bumpalo::Bump;
use lgn_ecs::prelude::ResMut;

use crate::{Handle, ObjectPool};

/// Handle around bumpalo allocator
pub type BumpAllocatorHandle = Handle<bumpalo::Bump>;

/// Pool of bump allocators. Creates new bump allocators if needed.
/// Meant to be used as a bevy resource.
pub struct BumpAllocatorPool {
    bump_allocator_pool: Mutex<ObjectPool<Bump>>,
}

impl BumpAllocatorPool {
    /// Creates an empty pool of bump allocators
    pub(crate) fn new() -> Self {
        Self {
            bump_allocator_pool: Mutex::new(ObjectPool::new()),
        }
    }

    /// Returns a handle to a bump allocator. Should be paired
    /// with `release_bump_allocator`
    pub fn acquire_bump_allocator(&self) -> BumpAllocatorHandle {
        let mut pool = self.bump_allocator_pool.lock().unwrap();
        pool.acquire_or_create(Bump::new)
    }

    /// Release bump allocator when done using it
    pub fn release_bump_allocator(&self, handle: BumpAllocatorHandle) {
        let mut pool = self.bump_allocator_pool.lock().unwrap();
        pool.release(handle);
    }

    /// Implements RAII pattern for acquiring and releasing of an allocator
    pub fn scoped_bump<F: FnOnce(&BumpAllocatorHandle)>(&self, f: F) {
        let bump = self.acquire_bump_allocator();
        f(&bump);
        self.release_bump_allocator(bump);
    }

    pub(crate) fn begin_frame(&mut self) {
        let mut pool = self.bump_allocator_pool.lock().unwrap();
        for bump in pool.iter_mut() {
            bump.reset();
        }
    }

    pub(crate) fn end_frame(&mut self) {
        let mut pool = self.bump_allocator_pool.lock().unwrap();
        pool.end_frame();
    }
}

pub(crate) fn begin_frame(mut bump_allocator_pool: ResMut<'_, BumpAllocatorPool>) {
    bump_allocator_pool.begin_frame();
}

pub(crate) fn end_frame(mut bump_allocator_pool: ResMut<'_, BumpAllocatorPool>) {
    bump_allocator_pool.end_frame();
}
