use lgn_graphics_api::{PagedBufferAllocation, Semaphore};

use crate::hl_gfx_api::HLQueue;

pub struct SparseBindingManager {
    sparse_buffer_bindings: Vec<PagedBufferAllocation>,
    sparse_buffer_unbindings: Vec<PagedBufferAllocation>,
}

impl SparseBindingManager {
    pub fn new() -> Self {
        Self {
            sparse_buffer_bindings: Vec::new(),
            sparse_buffer_unbindings: Vec::new(),
        }
    }

    pub fn add_sparse_binding(&mut self, binding: PagedBufferAllocation) {
        self.sparse_buffer_bindings.push(binding);
    }

    pub fn add_sparse_unbinding(&mut self, unbinding: PagedBufferAllocation) {
        self.sparse_buffer_unbindings.push(unbinding);
    }

    pub fn commmit_sparse_bindings<'a>(
        &mut self,
        queue: &HLQueue<'_>,
        prev_frame_semaphore: &'a Semaphore,
        unbind_semaphore: &'a Semaphore,
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        let result = queue.commmit_sparse_bindings(
            prev_frame_semaphore,
            &self.sparse_buffer_unbindings,
            unbind_semaphore,
            &self.sparse_buffer_bindings,
            bind_semaphore,
        );

        self.sparse_buffer_unbindings.clear();
        self.sparse_buffer_bindings.clear();

        result
    }
}
