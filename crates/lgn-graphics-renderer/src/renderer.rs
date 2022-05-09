#![allow(unsafe_code)]

use std::sync::Arc;

use lgn_graphics_api::{ApiDef, DeviceContext, Fence, FenceStatus, GfxApi};

use lgn_tracing::span_fn;

use crate::core::{RenderCommandBuilder, RenderCommandManager, RenderResources};

use crate::GraphicsQueue;

#[derive(Clone)]
pub struct GfxApiArc {
    inner: Arc<GfxApi>,
}

impl GfxApiArc {
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
    render_resources: RenderResources,
    _gfx_api: GfxApiArc,
}

impl Renderer {
    pub fn new(
        num_render_frames: u64,
        render_resources: RenderResources,
        gfx_api: GfxApiArc,
    ) -> Self {
        Self {
            num_render_frames,
            render_resources,
            _gfx_api: gfx_api.clone(),
        }
    }

    pub fn num_render_frames(&self) -> u64 {
        self.num_render_frames
    }

    pub(crate) fn render_resources(&self) -> &RenderResources {
        &self.render_resources
    }

    pub fn device_context(&self) -> DeviceContext {
        self.render_resources
            .get::<GfxApiArc>()
            .device_context()
            .clone()
    }

    pub fn render_command_builder(&self) -> RenderCommandBuilder {
        self.render_resources
            .get::<RenderCommandManager>()
            .command_builder()
    }

    pub fn graphics_queue(&self) -> GraphicsQueue {
        let graphics_queue = self.render_resources.get::<GraphicsQueue>();
        graphics_queue.clone()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue().queue_mut().wait_for_queue_idle();
    }
}

pub(crate) struct RenderScope {
    frame_idx: u64,
    render_frame_idx: u64,
    num_render_frames: u64,
    frame_fences: Vec<Fence>,
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
    }

    #[span_fn]
    pub(crate) fn end_frame(&mut self, graphics_queue: &GraphicsQueue) {
        let frame_fence = &self.frame_fences[self.render_frame_idx as usize];

        graphics_queue
            .queue_mut()
            .submit(&[], &[], &[], Some(frame_fence));
    }
}
