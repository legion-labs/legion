use std::sync::Arc;
use std::time;

use lgn_graphics_api::{ApiDef, DeviceContext, Fence, FenceStatus, GfxApi};

use lgn_tracing::span_fn;

use crate::core::{
    RenderCommandBuilder, RenderCommandQueuePool, RenderObject, RenderObjectAllocator,
    RenderObjects, RenderResources,
};

use crate::GraphicsQueue;

#[derive(Clone)]
pub struct GfxApiArc {
    inner: Arc<GfxApi>,
}

impl GfxApiArc {
    #[allow(unsafe_code)]
    pub fn new(api_def: ApiDef) -> Self {
        let gfx_api = unsafe { GfxApi::new(api_def).unwrap() };
        Self {
            inner: Arc::new(gfx_api),
        }
    }
}

impl std::ops::Deref for GfxApiArc {
    type Target = GfxApi;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Renderer {
    num_render_frames: u64,
    command_queue_pool: RenderCommandQueuePool,
    render_resources: RenderResources,
    graphics_queue: GraphicsQueue,
    gfx_api: GfxApiArc,
}

impl Renderer {
    pub fn new(
        num_render_frames: u64,
        command_queue_pool: RenderCommandQueuePool,
        render_resources: RenderResources,
        graphics_queue: GraphicsQueue,
        gfx_api: GfxApiArc,
    ) -> Self {
        Self {
            num_render_frames,
            command_queue_pool,
            render_resources,
            graphics_queue,
            gfx_api,
        }
    }

    pub fn num_render_frames(&self) -> u64 {
        self.num_render_frames
    }

    pub fn render_resources(&self) -> &RenderResources {
        &self.render_resources
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.gfx_api.device_context()
    }

    pub(crate) fn render_command_queue_pool(&mut self) -> &mut RenderCommandQueuePool {
        &mut self.command_queue_pool
    }

    pub fn render_command_builder(&self) -> RenderCommandBuilder {
        RenderCommandBuilder::new(&self.command_queue_pool)
    }

    pub fn graphics_queue(&self) -> &GraphicsQueue {
        &self.graphics_queue
    }

    pub fn allocate_render_object<F, R>(&self, func: F)
    where
        F: FnOnce(&mut RenderObjectAllocator<'_, R>),
        R: RenderObject,
    {
        let render_objects = self.render_resources.get::<RenderObjects>();
        let mut allocator = render_objects.create_allocator::<R>();
        func(&mut allocator);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue.queue_mut().wait_for_queue_idle();
    }
}

pub(crate) struct RenderScope {
    frame_idx: u64,
    render_frame_idx: u64,
    num_render_frames: u64,
    frame_fences: Vec<Fence>,
    frame_start: time::Instant,
    frame_time: time::Duration,
}

impl RenderScope {
    pub fn new(num_render_frames: u64, device_context: &DeviceContext) -> Self {
        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence())
                .collect(),
            frame_start: time::Instant::now(),
            frame_time: time::Duration::default(),
        }
    }

    #[span_fn]
    pub fn begin_frame(&mut self) {
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

        self.frame_start = time::Instant::now();
    }

    #[span_fn]
    pub(crate) fn end_frame(&mut self, graphics_queue: &GraphicsQueue) {
        let frame_fence = &self.frame_fences[self.render_frame_idx as usize];

        graphics_queue
            .queue_mut()
            .submit(&[], &[], &[], Some(frame_fence));

        let frame_end = time::Instant::now();
        self.frame_time = frame_end - self.frame_start;
    }

    pub(crate) fn frame_idx(&self) -> u64 {
        self.frame_idx
    }

    pub(crate) fn frame_time(&self) -> time::Duration {
        self.frame_time
    }
}
