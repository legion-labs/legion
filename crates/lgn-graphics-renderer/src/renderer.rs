use std::sync::Arc;
use std::time;

use lgn_graphics_api::{ApiDef, DeviceContext, Fence, FenceStatus, GfxApi};

use lgn_tracing::span_fn;

use crate::core::{RenderCommandBuilder, RenderCommandQueuePool, RenderResources};

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

    pub fn render_command_builder(&self) -> RenderCommandBuilder {
        self.command_queue_pool.builder()
    }

    pub fn graphics_queue(&self) -> &GraphicsQueue {
        &self.graphics_queue
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue.queue_mut().wait_for_queue_idle();
    }
}

pub struct BeginFrameClientFn {
    func: Box<dyn FnMut(&RenderResources, usize)>,
}

#[allow(unsafe_code)]
unsafe impl Send for BeginFrameClientFn {}

pub struct EndFrameClientFn {
    func: Box<dyn FnMut(&RenderResources, usize)>,
}

#[allow(unsafe_code)]
unsafe impl Send for EndFrameClientFn {}

pub(crate) struct RenderScope {
    frame_idx: u64,
    render_frame_idx: u64,
    num_render_frames: u64,
    frame_fences: Vec<Fence>,
    frame_start: time::Instant,
    frame_time: time::Duration,
    begin_frame_clients: Vec<BeginFrameClientFn>,
    end_frame_clients: Vec<EndFrameClientFn>,
}

pub(crate) struct RenderScopeBuilder {
    begin_frame_clients: Vec<BeginFrameClientFn>,
    end_frame_clients: Vec<EndFrameClientFn>,
}

impl RenderScopeBuilder {
    pub fn add_begin_frame<F: 'static>(mut self, f: F) -> Self
    where
        F: FnMut(&RenderResources, usize),
    {
        self.begin_frame_clients
            .push(BeginFrameClientFn { func: Box::new(f) });
        self
    }

    pub fn add_end_frame<F: 'static>(mut self, f: F) -> Self
    where
        F: FnMut(&RenderResources, usize),
    {
        self.end_frame_clients
            .push(EndFrameClientFn { func: Box::new(f) });
        self
    }

    pub fn build(self, num_render_frames: u64, device_context: &DeviceContext) -> RenderScope {
        RenderScope::new(
            num_render_frames,
            device_context,
            self.begin_frame_clients,
            self.end_frame_clients,
        )
    }
}

impl RenderScope {
    fn new(
        num_render_frames: u64,
        device_context: &DeviceContext,
        begin_frame_clients: Vec<BeginFrameClientFn>,
        end_frame_clients: Vec<EndFrameClientFn>,
    ) -> Self {
        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence())
                .collect(),
            frame_start: time::Instant::now(),
            frame_time: time::Duration::default(),
            begin_frame_clients,
            end_frame_clients,
        }
    }

    pub fn builder() -> RenderScopeBuilder {
        RenderScopeBuilder {
            begin_frame_clients: vec![],
            end_frame_clients: vec![],
        }
    }

    #[span_fn]
    pub fn begin_frame(&mut self, render_resources: &RenderResources) {
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

        for begin_frame_client in &mut self.begin_frame_clients {
            let func = &mut begin_frame_client.func;
            func(render_resources, self.frame_idx as usize);
        }
    }

    #[span_fn]
    pub(crate) fn end_frame(
        &mut self,
        render_resources: &RenderResources,
        graphics_queue: &GraphicsQueue,
    ) {
        for end_frame_client in &mut self.end_frame_clients {
            let func = &mut end_frame_client.func;
            func(render_resources, self.frame_idx as usize);
        }

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
