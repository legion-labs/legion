use std::ops::{Deref, DerefMut};

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
}

impl OnNewFrame for BumpAllocator {
    fn on_new_frame(&mut self) {
        self.bump_allocator.reset();
    }
}

impl Deref for BumpAllocator {
    type Target = bumpalo::Bump;
    fn deref(&self) -> &Self::Target {
        &self.bump_allocator
    }
}

impl DerefMut for BumpAllocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bump_allocator
    }
}
