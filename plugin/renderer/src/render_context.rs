use graphics_api::{DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetLayout, QueueType};
use graphics_utils::TransientBufferAllocator;

use crate::{
    CommandBufferHandle, CommandBufferPoolHandle, DescriptorHeapHandle, Renderer, RendererHandle,
};

// struct TransientDescriptorHeap {}

// // impl TransientDescriptorHeap {
// //     pub fn allocate_descriptor_set(
// //         &self,
// //         descriptor_set_layout: &DescriptorSetLayout,
// //     ) -> Result<DescriptorSetBufWriter> {
// //         Err(anyhow!("x"))
// //     }
// // }

type TransientBufferAllocatorHandle = RendererHandle<TransientBufferAllocator>;

pub struct RenderContext<'a> {
    renderer: &'a Renderer,
    cmd_buffer_pool_handle: CommandBufferPoolHandle,
    transient_descriptor_heap: DescriptorHeapHandle,
    transient_buffer_allocator: TransientBufferAllocatorHandle,
}

impl<'a> RenderContext<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let heap_def = default_descriptor_heap_size();
        Self {
            renderer,
            cmd_buffer_pool_handle: renderer.acquire_command_buffer_pool(QueueType::Graphics),
            transient_descriptor_heap: renderer.acquire_transient_descriptor_heap(&heap_def),
            transient_buffer_allocator: TransientBufferAllocatorHandle::new(TransientBufferAllocator::new(
                renderer.transient_buffer(),
                1000,
            )),
        }
    }

    pub fn renderer(&self) -> &'_ Renderer {
        self.renderer
    }

    // pub fn render_frame_idx(&self) -> usize {
    //     self.renderer.render_frame_idx()
    // }

    // pub fn queue(&self, queue_type: QueueType) -> &Queue {
    //     assert_eq!(queue_type, QueueType::Graphics);
    //     self.renderer.queue(queue_type)
    // }

    pub fn acquire_cmd_buffer(&mut self, queue_type: QueueType) -> CommandBufferHandle {
        assert_eq!(queue_type, QueueType::Graphics);
        self.cmd_buffer_pool_handle.acquire()
    }

    pub fn release_cmd_buffer(&mut self, handle: CommandBufferHandle) {
        self.cmd_buffer_pool_handle.release(handle);
    }

    pub fn alloc_descriptor_set(
        &mut self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> DescriptorSetBufWriter {
        if let Ok(writer) = self
            .transient_descriptor_heap
            .allocate_descriptor_set(descriptor_set_layout)
        {
            writer
        } else {
            self.renderer
                .release_transient_descriptor_heap(self.transient_descriptor_heap.take());
            let heap_def = default_descriptor_heap_size();
            self.transient_descriptor_heap =
                self.renderer.acquire_transient_descriptor_heap(&heap_def);
            self.transient_descriptor_heap
                .allocate_descriptor_set(descriptor_set_layout)
                .unwrap()
        }
    }

    pub fn acquire_transient_buffer_allocator(&mut self) -> TransientBufferAllocatorHandle {
        self.transient_buffer_allocator.take()
    }

    pub fn release_transient_buffer_allocator(&mut self, allocator: TransientBufferAllocatorHandle) {
        self.transient_buffer_allocator = allocator;
    }
}

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        self.renderer
            .release_command_buffer_pool(self.cmd_buffer_pool_handle.take());
        // match self.cmd_buffer_pool_handle.take() {
        //     Some(e) =>
        //     None => panic!(),
        // };

        self.renderer
            .release_transient_descriptor_heap(self.transient_descriptor_heap.take());

        self.transient_buffer_allocator.peek();
    }
}

fn default_descriptor_heap_size() -> DescriptorHeapDef {
    DescriptorHeapDef {
        transient: true,
        max_descriptor_sets: 4096,
        sampler_count: 128,
        constant_buffer_count: 1024,
        buffer_count: 1024,
        rw_buffer_count: 1024,
        texture_count: 1024,
        rw_texture_count: 1024,
    }
}
