use std::cell::RefCell;

use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorSetDataProvider, DescriptorSetHandle, DescriptorSetLayout,
    DescriptorSetWriter, Pipeline, QueueType, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use lgn_graphics_cgen_runtime::{PipelineDataProvider};

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
    descriptor_sets: RefCell<[Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS]>,
}

impl<'frame> RenderContext<'frame> {
    pub fn new(renderer: &'frame Renderer) -> Self {
        let heap_def = default_descriptor_heap_size();

        Self {
            renderer,
            cmd_buffer_pool: renderer.acquire_command_buffer_pool(QueueType::Graphics),
            descriptor_pool: renderer.acquire_descriptor_pool(&heap_def),
            transient_buffer_allocator: TransientBufferAllocatorHandle::new(
                TransientBufferAllocator::new(
                    renderer.device_context(),
                    &renderer.transient_buffer(),
                    1000,
                ),
            ),
            bump_allocator: renderer.acquire_bump_allocator(),
            descriptor_sets: RefCell::new([None; MAX_DESCRIPTOR_SET_LAYOUTS]),
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

    #[allow(clippy::todo)]
    pub fn write_descriptor_set(
        &self,
        descriptor_set: &impl DescriptorSetDataProvider,
    ) -> DescriptorSetHandle {
        let bump = self.bump_allocator().bumpalo();
        if let Ok(handle) = self
            .descriptor_pool
            .write_descriptor_set(descriptor_set, bump)
        {
            let mut descriptor_sets = self.descriptor_sets.borrow_mut();
            descriptor_sets[descriptor_set.layout().definition().frequency as usize] = Some(handle);
            handle
        } else {
            todo!("Descriptor OOM! ")
        }
    }

    pub fn populate_pipeline_data(&self, pipeline_data: &mut impl PipelineDataProvider) {
        let descriptor_sets = self.descriptor_sets.borrow();
        for i in 0..MAX_DESCRIPTOR_SET_LAYOUTS as u32 {
            pipeline_data.set_descriptor_set(i, descriptor_sets[i as usize]);
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
