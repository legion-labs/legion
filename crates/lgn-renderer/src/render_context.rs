use lgn_core::Handle;
use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorRef, DescriptorSetHandle, DescriptorSetLayout, QueueType,
};

use crate::{
    hl_gfx_api::{HLCommandBuffer, HLQueue},
    resources::{
        CommandBufferPoolHandle, DescriptorHeapManager, DescriptorPoolHandle, PipelineManager,
        TransientBufferAllocator,
    },
    Renderer,
};

pub(crate) type TransientBufferAllocatorHandle = Handle<TransientBufferAllocator>;

pub struct RenderContext<'frame> {
    renderer: &'frame Renderer,
    descriptor_heap_manager: &'frame DescriptorHeapManager,
    pipeline_manager: &'frame PipelineManager,
    cmd_buffer_pool: CommandBufferPoolHandle,
    descriptor_pool: DescriptorPoolHandle,
    transient_buffer_allocator: TransientBufferAllocatorHandle,
    // tmp
    frame_descriptor_set: Option<(&'frame DescriptorSetLayout, DescriptorSetHandle)>,
    view_descriptor_set: Option<(&'frame DescriptorSetLayout, DescriptorSetHandle)>,
}

impl<'frame> RenderContext<'frame> {
    pub fn new(
        renderer: &'frame Renderer,
        descriptor_heap_manager: &'frame DescriptorHeapManager,
        pipeline_manager: &'frame PipelineManager,
    ) -> Self {
        let heap_def = default_descriptor_heap_size();

        Self {
            renderer,
            pipeline_manager,
            descriptor_heap_manager,
            cmd_buffer_pool: renderer.acquire_command_buffer_pool(QueueType::Graphics),
            descriptor_pool: descriptor_heap_manager.acquire_descriptor_pool(&heap_def),
            transient_buffer_allocator: TransientBufferAllocatorHandle::new(
                TransientBufferAllocator::new(
                    renderer.device_context(),
                    &renderer.transient_buffer(),
                    1000,
                ),
            ),
            frame_descriptor_set: None,
            view_descriptor_set: None,
        }
    }

    pub fn renderer(&self) -> &Renderer {
        self.renderer
    }

    pub fn pipeline_manager(&self) -> &PipelineManager {
        self.pipeline_manager
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
    pub fn write_descriptor_set(
        &self,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef<'_>],
    ) -> DescriptorSetHandle {
        self.descriptor_pool
            .write_descriptor_set(layout, descriptors)
    }

    pub(crate) fn transient_buffer_allocator(&self) -> &TransientBufferAllocatorHandle {
        &self.transient_buffer_allocator
    }

    pub fn frame_descriptor_set(&self) -> (&DescriptorSetLayout, DescriptorSetHandle) {
        self.frame_descriptor_set.unwrap()
    }

    pub fn set_frame_descriptor_set(
        &mut self,
        layout: &'frame DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.frame_descriptor_set = Some((layout, handle));
    }

    pub fn view_descriptor_set(&self) -> (&DescriptorSetLayout, DescriptorSetHandle) {
        self.view_descriptor_set.unwrap()
    }

    pub fn set_view_descriptor_set(
        &mut self,
        layout: &'frame DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.view_descriptor_set = Some((layout, handle));
    }
}

impl<'frame> Drop for RenderContext<'frame> {
    fn drop(&mut self) {
        self.renderer
            .release_command_buffer_pool(self.cmd_buffer_pool.transfer());

        self.descriptor_heap_manager
            .release_descriptor_pool(self.descriptor_pool.transfer());

        self.transient_buffer_allocator.take();
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
