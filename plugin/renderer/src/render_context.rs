use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorSetLayout, DescriptorSetWriter, QueueType,
};
use lgn_graphics_cgen_runtime::CGenRuntime;

use crate::{
    memory::BumpAllocatorHandle,
    resources::{
        CommandBufferHandle, CommandBufferPoolHandle, DescriptorPool, DescriptorPoolHandle,
        TransientBufferAllocator,
    },
    RenderHandle, Renderer,
};

type TransientBufferAllocatorHandle = RenderHandle<TransientBufferAllocator>;

pub struct RenderContext<'frame> {
    renderer: &'frame Renderer,
    cgen_runtime: CGenRuntime,
    cmd_buffer_pool_handle: CommandBufferPoolHandle,
    descriptor_pool: DescriptorPoolHandle,
    transient_buffer_allocator: TransientBufferAllocatorHandle,
    bump_allocator: BumpAllocatorHandle,
}

impl<'frame> RenderContext<'frame> {
    pub fn new(renderer: &'frame Renderer) -> Self {
        let heap_def = default_descriptor_heap_size();

        Self {
            renderer,
            cgen_runtime: renderer.cgen_runtime().clone(),
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

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn cgen_runtime(&self) -> &CGenRuntime {
        &self.cgen_runtime
    }

    pub fn acquire_cmd_buffer(&mut self, queue_type: QueueType) -> CommandBufferHandle {
        assert_eq!(queue_type, QueueType::Graphics);
        self.cmd_buffer_pool_handle.acquire()
    }

    pub fn release_cmd_buffer(&mut self, handle: CommandBufferHandle) {
        self.cmd_buffer_pool_handle.release(handle);
    }

    pub fn descriptor_pool(&self) -> &DescriptorPoolHandle {
        &self.descriptor_pool
    }

    // pub fn acquire_descriptor_pool(&mut self) -> DescriptorPoolHandle {
    //     self.descriptor_pool.transfer()
    // }

    // pub fn release_descriptor_pool(&mut self, handle: DescriptorPoolHandle) {
    //     self.descriptor_pool = handle;
    // }

    #[allow(unreachable_code)]
    pub fn alloc_descriptor_set(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> DescriptorSetWriter {
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

    pub(crate) fn acquire_transient_buffer_allocator(&mut self) -> TransientBufferAllocatorHandle {
        self.transient_buffer_allocator.transfer()
    }

    pub fn release_transient_buffer_allocator(&mut self, handle: TransientBufferAllocatorHandle) {
        self.transient_buffer_allocator = handle;
    }

    pub fn bump_allocator(&self) -> &BumpAllocatorHandle {
        &self.bump_allocator
    }

    // pub fn acquire_bump_allocator(&mut self) -> BumpAllocatorHandle {
    //     self.bump_allocator.transfer()
    // }

    // pub fn release_bump_allocator(&mut self, handle: BumpAllocatorHandle) {
    //     self.bump_allocator = handle;
    // }
}

impl<'frame> Drop for RenderContext<'frame> {
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
