#![allow(clippy::pedantic)]
use std::convert::TryFrom;

use graphics_api::prelude::*;
use lgn_presenter::swapchain_helper::SwapchainHelper;
use lgn_renderer::{
    components::{Presenter, RenderSurface, RenderSurfaceExtents},
    RenderContext, Renderer,
};
use lgn_tasks::TaskPool;
use raw_window_handle::HasRawWindowHandle;

pub struct PresenterWindow {
    swapchain_helper: SwapchainHelper,
    extents: RenderSurfaceExtents,
}

impl std::fmt::Debug for PresenterWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterWindow").finish()
    }
}

impl PresenterWindow {
    pub fn from_window(
        renderer: &Renderer,
        hwnd: &dyn HasRawWindowHandle,
        extents: RenderSurfaceExtents,
    ) -> Self {
        let device_context = renderer.device_context();
        let swapchain = device_context
            .create_swapchain(
                hwnd,
                &SwapchainDef {
                    width: extents.width(),
                    height: extents.height(),
                    enable_vsync: true,
                },
            )
            .unwrap();

        Self {
            swapchain_helper: SwapchainHelper::new(device_context, swapchain, None).unwrap(),
            extents,
        }
    }

    pub fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
    ) {
        //
        // Acquire backbuffer
        //
        let extents = self.extents;
        let presentable_frame = self
            .swapchain_helper
            .acquire_next_image(extents.width(), extents.height(), None)
            .unwrap();

        //
        // Blit render surface
        //
        let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);
        let swapchain_texture = presentable_frame.swapchain_texture();

        cmd_buffer.begin().unwrap();

        {
            render_surface.transition_to(&cmd_buffer, ResourceState::COPY_SRC);

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::state_transition(
                        swapchain_texture,
                        ResourceState::PRESENT,
                        ResourceState::COPY_DST,
                    )],
                )
                .unwrap();

            let src_texture = render_surface.texture();
            let src_texture_def = src_texture.definition();
            let dst_texture = swapchain_texture;
            let dst_texture_def = dst_texture.definition();
            let blit_params = CmdBlitParams {
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::COPY_DST,
                src_offsets: [
                    Offset3D { x: 0, y: 0, z: 0 },
                    Offset3D {
                        x: i32::try_from(src_texture_def.extents.width).unwrap(),
                        y: i32::try_from(src_texture_def.extents.height).unwrap(),
                        z: 1,
                    },
                ],
                dst_offsets: [
                    Offset3D { x: 0, y: 0, z: 0 },
                    Offset3D {
                        x: i32::try_from(dst_texture_def.extents.width).unwrap(),
                        y: i32::try_from(dst_texture_def.extents.height).unwrap(),
                        z: 1,
                    },
                ],
                src_mip_level: 0,
                dst_mip_level: 0,
                array_slices: Some([0, 0]),
                filtering: FilterType::Linear,
            };
            cmd_buffer
                .cmd_blit_texture(src_texture, dst_texture, &blit_params)
                .unwrap();

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::state_transition(
                        swapchain_texture,
                        ResourceState::COPY_DST,
                        ResourceState::PRESENT,
                    )],
                )
                .unwrap();
        }

        cmd_buffer.end().unwrap();

        //
        // Present
        //
        {
            let renderer = render_context.renderer();
            let present_queue = renderer.queue(QueueType::Graphics);
            let wait_sem = render_surface.sema();
            presentable_frame
                .present(&present_queue, wait_sem, &[&cmd_buffer])
                .unwrap();
        }

        render_context.release_cmd_buffer(cmd_buffer);
    }
}

impl Drop for PresenterWindow {
    fn drop(&mut self) {
        self.swapchain_helper.destroy(None).unwrap();
    }
}

impl Presenter for PresenterWindow {
    fn resize(&mut self, _renderer: &Renderer, extents: RenderSurfaceExtents) {
        self.extents = extents;
    }

    fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
        _task_pool: &TaskPool,
    ) {
        // FIXME: if the windows is minimized, we should not resize the RenderSurface and we should not present
        // the swapchain.
        if self.extents.width() > 1 && self.extents.height() > 1 {
            self.present(render_context, render_surface);
        }
    }
}
