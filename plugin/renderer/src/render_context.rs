use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorSetDataProvider, DescriptorSetHandle, DescriptorSetLayout,
    DescriptorSetWriter, QueueType,
};
use lgn_graphics_cgen_runtime::CGenRuntime;

use crate::{
    hl_gfx_api::{HLCommandBuffer, HLQueue},
    memory::BumpAllocatorHandle,
    resources::{CommandBufferPoolHandle, DescriptorPoolHandle, TransientBufferAllocator},
    RenderHandle, Renderer,
};

pub(crate) type TransientBufferAllocatorHandle = RenderHandle<TransientBufferAllocator>;

pub struct RenderContext<'frame> {
    renderer: &'frame Renderer,
    cmd_buffer_pool: CommandBufferPoolHandle,
    descriptor_pool: DescriptorPoolHandle,
    transient_buffer_allocator: TransientBufferAllocatorHandle,
    bump_allocator: BumpAllocatorHandle,
}

impl<'frame> RenderContext<'frame> {
    pub fn new(renderer: &'frame Renderer) -> Self {
        let heap_def = default_descriptor_heap_size();

        Self {
            renderer,
            cmd_buffer_pool: renderer.acquire_command_buffer_pool(QueueType::Graphics),
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

    pub fn renderer(&self) -> &Renderer {
        self.renderer
    }

    pub fn graphics_queue(&self) -> HLQueue<'_> {
        HLQueue::new(
            self.renderer.graphics_queue_guard(QueueType::Graphics),
            &self.cmd_buffer_pool,
        )
    }

    pub fn alloc_command_buffer(&self) -> HLCommandBuffer<'_> {
        HLCommandBuffer::new(&self.cmd_buffer_pool)
    }

    pub fn descriptor_pool(&self) -> &DescriptorPoolHandle {
        &self.descriptor_pool
    }

    #[allow(clippy::todo)]
    pub fn alloc_descriptor_set(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> DescriptorSetWriter<'_> {
        let bump = self.bump_allocator().bumpalo();
        if let Ok(writer) = self
            .descriptor_pool
            .allocate_descriptor_set(descriptor_set_layout, bump)
        {
            writer
        } else {
            todo!("Descriptor OOM! ")
        }
    }

    pub fn write_descriptor_set(
        &self,
        descriptor_set: &impl DescriptorSetDataProvider,
    ) -> DescriptorSetHandle {
        let bump = self.bump_allocator().bumpalo();
        if let Ok(handle) = self
            .descriptor_pool
            .write_descriptor_set(descriptor_set, bump)
        {
            handle
        } else {
            todo!("Descriptor OOM! ")
        }
    }

    pub(crate) fn transient_buffer_allocator(&self) -> &TransientBufferAllocatorHandle {
        &self.transient_buffer_allocator
    }

    pub fn bump_allocator(&self) -> &BumpAllocatorHandle {
        &self.bump_allocator
    }
}

impl<'frame> Drop for RenderContext<'frame> {
    fn drop(&mut self) {
        self.renderer
            .release_command_buffer_pool(self.cmd_buffer_pool.transfer());

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
