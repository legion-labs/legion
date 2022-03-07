#![allow(unsafe_code)]

use lgn_core::Handle;
use lgn_graphics_api::Queue;
use lgn_graphics_api::{
    ApiDef, BufferView, DeviceContext, Fence, FenceStatus, GfxApi, QueueType, Semaphore,
};

use lgn_tracing::span_fn;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};

use crate::cgen::cgen_type::{DirectionalLight, OmniDirectionalLight, SpotLight};

use crate::resources::{
    CommandBufferPool, CommandBufferPoolHandle, GpuSafePool, TransientPagedBuffer,
    UnifiedStaticBuffer, UnifiedStaticBufferAllocator, UniformGPUData,
    UniformGPUDataUploadJobBlock,
};
use crate::RenderContext;

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    prev_frame_sems: Vec<Semaphore>,
    sparse_unbind_sems: Vec<Semaphore>,
    sparse_bind_sems: Vec<Semaphore>,
    frame_fences: Vec<Fence>,
    graphics_queue: RwLock<Queue>,
    command_buffer_pools: Mutex<GpuSafePool<CommandBufferPool>>,
    transient_buffer: TransientPagedBuffer,
    static_buffer: UnifiedStaticBuffer,
    omnidirectional_lights_data: OmniDirectionalLightsStaticBuffer,
    directional_lights_data: DirectionalLightsStaticBuffer,
    spotlights_data: SpotLightsStaticBuffer,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

pub type OmniDirectionalLightsStaticBuffer = Handle<UniformGPUData<OmniDirectionalLight>>;
pub type DirectionalLightsStaticBuffer = Handle<UniformGPUData<DirectionalLight>>;
pub type SpotLightsStaticBuffer = Handle<UniformGPUData<SpotLight>>;

macro_rules! impl_static_buffer_accessor {
    ($name:ident, $buffer_type:ty, $type:ty) => {
        paste::paste! {
            pub fn [<acquire_ $name>](&mut self) -> $buffer_type {
                self.$name.transfer()
            }
            pub fn [<release_ $name>](&mut self, $name: $buffer_type) {
                self.$name = $name;
            }
            pub fn [<$name _structured_buffer_view>](&self) -> BufferView{
                self.$name.structured_buffer_view($type::SIZE as u64)
            }
        }
    };
}

impl Renderer {
    pub fn new(num_render_frames: usize) -> Self {
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();

        let static_buffer = UnifiedStaticBuffer::new(device_context, 64 * 1024 * 1024, false);

        let omnidirectional_lights_data =
            OmniDirectionalLightsStaticBuffer::new(UniformGPUData::<OmniDirectionalLight>::new(
                Some(static_buffer.allocator()),
                OmniDirectionalLight::PAGE_SIZE,
            ));

        let directional_lights_data =
            DirectionalLightsStaticBuffer::new(UniformGPUData::<DirectionalLight>::new(
                Some(static_buffer.allocator()),
                DirectionalLight::PAGE_SIZE,
            ));

        let spotlights_data = SpotLightsStaticBuffer::new(UniformGPUData::<SpotLight>::new(
            Some(static_buffer.allocator()),
            SpotLight::PAGE_SIZE,
        ));

        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            prev_frame_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            sparse_unbind_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            sparse_bind_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
                .collect(),
            graphics_queue: RwLock::new(device_context.create_queue(QueueType::Graphics).unwrap()),

            command_buffer_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
            transient_buffer: TransientPagedBuffer::new(device_context, 512, 64 * 1024),
            static_buffer,
            omnidirectional_lights_data,
            directional_lights_data,
            spotlights_data,
            api,
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.api.device_context()
    }

    pub fn num_render_frames(&self) -> usize {
        self.num_render_frames
    }

    pub fn render_frame_idx(&self) -> usize {
        self.render_frame_idx
    }

    pub fn graphics_queue_guard(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            _ => unreachable!(),
        }
    }

    // TMP: change that.
    pub(crate) fn transient_buffer(&self) -> TransientPagedBuffer {
        self.transient_buffer.clone()
    }

    impl_static_buffer_accessor!(
        omnidirectional_lights_data,
        OmniDirectionalLightsStaticBuffer,
        OmniDirectionalLight
    );

    impl_static_buffer_accessor!(
        directional_lights_data,
        DirectionalLightsStaticBuffer,
        DirectionalLight
    );

    impl_static_buffer_accessor!(spotlights_data, SpotLightsStaticBuffer, SpotLight);

    pub fn static_buffer(&self) -> &UnifiedStaticBuffer {
        &self.static_buffer
    }

    pub fn static_buffer_allocator(&self) -> &UnifiedStaticBufferAllocator {
        self.static_buffer.allocator()
    }

    pub fn add_update_job_block(&self, job_blocks: &mut Vec<UniformGPUDataUploadJobBlock>) {
        self.static_buffer
            .allocator()
            .add_update_job_block(job_blocks);
    }

    #[span_fn]
    pub fn flush_update_jobs(&self, render_context: &RenderContext<'_>) {
        let prev_frame_semaphore = &self.prev_frame_sems[self.render_frame_idx];
        let unbind_semaphore = &self.sparse_unbind_sems[self.render_frame_idx];
        let bind_semaphore = &self.sparse_bind_sems[self.render_frame_idx];

        self.static_buffer.allocator().flush_updater(
            prev_frame_semaphore,
            unbind_semaphore,
            bind_semaphore,
            render_context,
        );
    }

    pub fn static_buffer_ro_view(&self) -> BufferView {
        self.static_buffer.read_only_view()
    }

    //    pub fn prev_frame_semaphore(&self)

    pub(crate) fn acquire_command_buffer_pool(
        &self,
        queue_type: QueueType,
    ) -> CommandBufferPoolHandle {
        let queue = self.graphics_queue_guard(queue_type);
        let mut pool = self.command_buffer_pools.lock();
        pool.acquire_or_create(|| CommandBufferPool::new(&*queue))
    }

    pub(crate) fn release_command_buffer_pool(&self, handle: CommandBufferPoolHandle) {
        let mut pool = self.command_buffer_pools.lock();
        pool.release(handle);
    }

    #[span_fn]
    pub(crate) fn begin_frame(&mut self) {
        //
        // Update frame indices
        //
        self.frame_idx += 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;

        //
        // Wait for the next cpu frame to be available
        //
        let signal_fence = &self.frame_fences[self.render_frame_idx];
        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

        //
        // Now, it is safe to free memory
        //
        let device_context = self.api.device_context();
        device_context.free_gpu_memory();

        //
        // Broadcast begin frame event
        //
        {
            let mut pool = self.command_buffer_pools.lock();
            pool.begin_frame();
        }

        // TMP: todo
        self.transient_buffer.begin_frame();
    }

    #[span_fn]
    pub(crate) fn end_frame(&mut self) {
        let graphics_queue = self.graphics_queue.write();
        let frame_fence = &self.frame_fences[self.render_frame_idx];

        graphics_queue
            .submit(&[], &[], &[], Some(frame_fence))
            .unwrap();

        //
        // Broadcast end frame event
        //

        {
            let mut pool = self.command_buffer_pools.lock();
            pool.end_frame();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        {
            let  graphics_queue = self.graphics_queue_guard(QueueType::Graphics);
            graphics_queue.wait_for_queue_idle().unwrap();
        }
        std::mem::drop(self.spotlights_data.take());
        std::mem::drop(self.directional_lights_data.take());
        std::mem::drop(self.omnidirectional_lights_data.take());
    }
}
