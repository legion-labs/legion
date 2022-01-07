#![allow(clippy::inline_always)]

use crate::{resources::OnFrameEventHandler, RenderHandle};

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

    pub fn bumpalo(&self) -> &bumpalo::Bump {
        &self.bump_allocator
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump_allocator.alloc(val)
    }

    pub fn bump(&self) -> &bumpalo::Bump {
        &self.bump_allocator
    }
}

impl OnFrameEventHandler for BumpAllocator {
    fn on_begin_frame(&mut self) {
        self.bump_allocator.reset();
    }

    fn on_end_frame(&mut self) {}
}
