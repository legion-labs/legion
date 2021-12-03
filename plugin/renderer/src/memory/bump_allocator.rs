#![allow(clippy::inline_always)]

use crate::{resources::OnNewFrame, RenderHandle};

pub struct BumpAllocator {
    bump_allocator: bumpalo::Bump,
}

pub type BumpAllocatorHandle = RenderHandle<BumpAllocator>;

impl BumpAllocator {
    pub(crate) fn new() -> Self {
        Self {
            bump_allocator: bumpalo::Bump::new(),
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump_allocator.alloc(val)
    }
}

impl OnNewFrame for BumpAllocator {
    fn on_new_frame(&mut self) {
        self.bump_allocator.reset();
    }
}
