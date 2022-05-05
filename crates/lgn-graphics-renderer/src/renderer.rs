#![allow(unsafe_code)]

use lgn_graphics_api::{DeviceContext, Fence, FenceStatus, GfxApi, QueueType};

use lgn_tracing::span_fn;

use crate::core::{RenderCommandBuilder, RenderCommandManager, RenderResources};
use crate::resources::{CommandBufferPoolHandle, TransientCommandBufferManager};
use crate::GraphicsQueue;

pub struct Renderer {
    num_render_frames: u64,
    render_resources: RenderResources,
}

impl Renderer {
    pub fn new(num_render_frames: u64, render_resources: RenderResources) -> Self {
        Self {
            num_render_frames,
            render_resources,
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
            .get::<GfxApi>()
            .device_context()
            .clone()
    }

    pub fn render_command_builder(&self) -> RenderCommandBuilder {
        self.render_resources
            .get::<RenderCommandManager>()
            .builder()
    }

    pub fn graphics_queue(&self) -> GraphicsQueue {
        let graphics_queue = self.render_resources.get::<GraphicsQueue>();
        graphics_queue.clone()
    }

    pub(crate) fn acquire_command_buffer_pool(
        &self,
        queue_type: QueueType,
    ) -> CommandBufferPoolHandle {
        assert!(queue_type == QueueType::Graphics);

        let command_buffer_manager = self.render_resources.get::<TransientCommandBufferManager>();
        command_buffer_manager.acquire()
    }

    pub(crate) fn release_command_buffer_pool(&self, handle: CommandBufferPoolHandle) {
        let command_buffer_manager = self.render_resources.get::<TransientCommandBufferManager>();
        command_buffer_manager.release(handle);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue().queue().wait_for_queue_idle().unwrap();
    }
}

pub(crate) struct RendererData {
    frame_idx: u64,
    render_frame_idx: u64,
    num_render_frames: u64,
    frame_fences: Vec<Fence>,
}

impl RendererData {
    pub fn new(num_render_frames: u64, device_context: &DeviceContext) -> Self {
        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
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
            .queue()
            .submit(&mut [], &[], &[], Some(frame_fence))
            .unwrap();
    }
}
