#![allow(unsafe_code)]

use lgn_graphics_api::Queue;
use lgn_graphics_api::{ApiDef, BufferView, DeviceContext, Fence, FenceStatus, GfxApi, QueueType};

use lgn_tracing::span_fn;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};

use crate::resources::{
    CommandBufferPool, CommandBufferPoolHandle, GPUDataUpdaterCopy, GpuSafePool,
    TransientBufferAllocator, TransientPagedBuffer, UnifiedStaticBuffer,
    UnifiedStaticBufferAllocator,
};
use crate::RenderContext;

pub struct Renderer {
    frame_idx: u64,
    render_frame_idx: u64,
    num_render_frames: u64,
    frame_fences: Vec<Fence>,
    graphics_queue: RwLock<Queue>,
    command_buffer_pools: Mutex<GpuSafePool<CommandBufferPool>>,
    transient_buffer: TransientPagedBuffer,
    static_buffer: UnifiedStaticBuffer,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

impl Renderer {
    pub fn new(num_render_frames: u64) -> Self {
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();

        let static_buffer = UnifiedStaticBuffer::new(device_context, 64 * 1024 * 1024);

        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
                .collect(),
            graphics_queue: RwLock::new(device_context.create_queue(QueueType::Graphics).unwrap()),
            command_buffer_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
            transient_buffer: TransientPagedBuffer::new(device_context, num_render_frames),
            static_buffer,
            api,
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.api.device_context()
    }

    pub fn num_render_frames(&self) -> u64 {
        self.num_render_frames
    }

    pub fn render_frame_idx(&self) -> u64 {
        self.render_frame_idx
    }

    pub fn graphics_queue_guard(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn transient_buffer_allocator(
        &self,
        min_alloc_size: u64,
    ) -> TransientBufferAllocator {
        TransientBufferAllocator::new(
            self.device_context(),
            &self.transient_buffer,
            min_alloc_size,
        )
    }

    pub fn static_buffer(&self) -> &UnifiedStaticBuffer {
        &self.static_buffer
    }

    pub fn static_buffer_allocator(&self) -> &UnifiedStaticBufferAllocator {
        self.static_buffer.allocator()
    }

    pub fn add_update_job_block(&self, job_blocks: Vec<GPUDataUpdaterCopy>) {
        self.static_buffer
            .allocator()
            .add_update_job_block(job_blocks);
    }

    #[span_fn]
    pub fn flush_update_jobs(&self, render_context: &RenderContext<'_>) {
        self.static_buffer.allocator().flush_updater(render_context);
    }

    pub fn static_buffer_ro_view(&self) -> &BufferView {
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
        self.render_frame_idx = self.frame_idx % self.num_render_frames as u64;

        //
        // Wait for the next cpu frame to be available
        //
        let signal_fence = &self.frame_fences[self.render_frame_idx as usize];
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
            pool.begin_frame(|x| x.begin_frame());

            self.transient_buffer.begin_frame();
        }

        //
        // Update the current frame used for timeline semaphores
        //
        self.api.device_context_mut().inc_current_cpu_frame();
    }

    #[span_fn]
    pub(crate) fn end_frame(&mut self) {
        let graphics_queue = self.graphics_queue.write();
        let frame_fence = &self.frame_fences[self.render_frame_idx as usize];

        graphics_queue
            .submit(&mut [], &[], &[], Some(frame_fence))
            .unwrap();

        //
        // Broadcast end frame event
        //
        {
            self.transient_buffer.end_frame();

            let mut pool = self.command_buffer_pools.lock();
            pool.end_frame(|x| x.end_frame());
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        {
            let graphics_queue = self.graphics_queue_guard(QueueType::Graphics);
            graphics_queue.wait_for_queue_idle().unwrap();
        }
    }
}
