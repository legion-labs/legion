use lgn_graphics_api::{DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetLayout, QueueType};

use crate::{
    memory::BumpAllocatorHandle,
    resources::{
        CommandBufferHandle, CommandBufferPoolHandle, DescriptorPoolHandle,
        TransientBufferAllocator,
    },
    RenderHandle, Renderer,
};

type TransientBufferAllocatorHandle = RenderHandle<TransientBufferAllocator>;

pub struct RenderContext<'a> {
    renderer: &'a Renderer,
    cmd_buffer_pool_handle: CommandBufferPoolHandle,
    descriptor_pool: DescriptorPoolHandle,
    transient_buffer_allocator: TransientBufferAllocatorHandle,
    bump_allocator: BumpAllocatorHandle,
}

impl<'a> RenderContext<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let heap_def = default_descriptor_heap_size();
        Self {
            renderer,
            cmd_buffer_pool_handle: renderer.acquire_command_buffer_pool(QueueType::Graphics),
            descriptor_pool: renderer.acquire_descriptor_pool(&heap_def),
            // TMP: we should acquire a handle from the renderer
            transient_buffer_allocator: TransientBufferAllocatorHandle::new(
                TransientBufferAllocator::new(
                    renderer.device_context(),
                    &renderer.transient_buffer(),
                    1000,
                ),
            ),
            bump_allocator: renderer.acquire_bump_allocator(),
        }
    }

    pub fn renderer(&self) -> &'_ Renderer {
        self.renderer
    }

    pub fn acquire_cmd_buffer(&mut self, queue_type: QueueType) -> CommandBufferHandle {
        assert_eq!(queue_type, QueueType::Graphics);
        self.cmd_buffer_pool_handle.acquire()
    }

    pub fn release_cmd_buffer(&mut self, handle: CommandBufferHandle) {
        self.cmd_buffer_pool_handle.release(handle);
    }

    #[allow(unreachable_code)]
    pub fn alloc_descriptor_set(
        &mut self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> DescriptorSetBufWriter {
        if let Ok(writer) = self
            .descriptor_pool
            .allocate_descriptor_set(descriptor_set_layout)
        {
            writer
        } else {
            todo!("Descriptor OOM! ")
        }
    }

    pub(crate) fn acquire_transient_buffer_allocator(&mut self) -> TransientBufferAllocatorHandle {
        self.transient_buffer_allocator.transfer()
    }

    pub fn release_transient_buffer_allocator(&mut self, handle: TransientBufferAllocatorHandle) {
        self.transient_buffer_allocator = handle;
    }

    pub fn acquire_bump_allocator(&mut self) -> BumpAllocatorHandle {
        self.bump_allocator.transfer()
    }

    pub fn release_bump_allocator(&mut self, handle: BumpAllocatorHandle) {
        self.bump_allocator = handle;
    }
}

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        self.renderer
            .release_command_buffer_pool(self.cmd_buffer_pool_handle.transfer());

        self.renderer
            .release_descriptor_pool(self.descriptor_pool.transfer());

        self.transient_buffer_allocator.take();

        self.renderer
            .release_bump_allocator(self.bump_allocator.transfer());
    }
}

fn default_descriptor_heap_size() -> DescriptorHeapDef {
    DescriptorHeapDef {
        max_descriptor_sets: 4096,
        sampler_count: 128,
        constant_buffer_count: 1024,
        buffer_count: 1024,
        rw_buffer_count: 1024,
        texture_count: 1024,
        rw_texture_count: 1024,
    }
}
