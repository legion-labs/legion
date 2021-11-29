#![allow(clippy::pedantic)]

use std::convert::TryFrom;

use graphics_api::prelude::*;

use legion_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    Presenter, RenderContext, Renderer,
};
use legion_window::WindowId;
use raw_window_handle::HasRawWindowHandle;

use legion_presenter::swapchain_helper::SwapchainHelper;

// #[derive(Component)]
pub struct PresenterWindow {
    window_id: WindowId,
    // render_surface_id: RenderSurfaceId,
    swapchain_helper: SwapchainHelper,
    extents: RenderSurfaceExtents, //  Extents2D, // cmd_pools: Vec<CommandPool>,
                                   // cmd_buffers: Vec<CommandBuffer>
}

impl std::fmt::Debug for PresenterWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterWindow").finish()
    }
}

impl PresenterWindow {
    pub fn from_window(
        renderer: &Renderer,
        window_id: WindowId,
        hwnd: &dyn HasRawWindowHandle,
        // render_surface_id: RenderSurfaceId,
        extents: RenderSurfaceExtents,
    ) -> Self {
        // let extents = Self::get_window_extents_from_wnd(wnd);
        let device_context = renderer.device_context();
        // let present_queue = renderer.queue(QueueType::Graphics);
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

        // let swapchain_helper = ;
        // let mut cmd_pools = Vec::with_capacity(swapchain_helper.image_count());
        // let mut cmd_buffers = Vec::with_capacity(swapchain_helper.image_count());
        // for _ in 0..swapchain_helper.image_count() {
        //     let cmd_pool = present_queue
        //         .create_command_pool(&CommandPoolDef { transient: true })
        //         .unwrap();

        //     let cmd_buffer = cmd_pool
        //         .create_command_buffer(&CommandBufferDef {
        //             is_secondary: false,
        //         })
        //         .unwrap();

        //     cmd_pools.push(cmd_pool);
        //     cmd_buffers.push(cmd_buffer);
        // }

        Self {
            // window_id: wnd.id(),
            window_id,
            // render_surface_id,
            swapchain_helper: SwapchainHelper::new(device_context, swapchain, None).unwrap(),
            extents// : Self::get_window_extents_from_wnd(wnd)            // cmd_pools,
            // cmd_buffers,
        }
    }

    pub fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        // wnd: &Window,
        // present_queue: &mut Queue,
        render_surface: &mut RenderSurface,
    ) {
        //
        // Acquire backbuffer
        //
        // let extents = Self::get_window_extents(wnd);
        let extents = self.extents;
        let presentable_frame = self
            .swapchain_helper
            .acquire_next_image(extents.width(), extents.height(), None)
            .unwrap();

        //
        // Blit render surface
        //
        let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);
        // let cmd_buffer = {

        let swapchain_texture = presentable_frame.swapchain_texture();
        // let cmd_pool = &self.cmd_pools[presentable_frame.rotating_frame_index()];
        // let cmd_buffer = &self.cmd_buffers[presentable_frame.rotating_frame_index()];

        // cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();

        // Option<&mut RenderSurface>
        // if let Some(render_surface) = &mut render_surface
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
        // let wait_sem = render_surface.map(|e| e.sema());
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

    pub fn window_id(&self) -> WindowId {
        self.window_id
    }

    // pub fn render_surface_id(&self) -> RenderSurfaceId {
    //     self.render_surface_id
    // }

    // fn get_window_extents_from_wnd(wnd: &Window) -> Extents2D {
    //     Self::get_window_extents(wnd.physical_width(), wnd.physical_height())
    // }

    // fn get_window_extents(width: u32, height: u32) -> Extents2D {
    //     Extents2D {
    //         width: max(1u32, width),
    //         height: max(1u32, height),
    //     }
    // }
}

impl Drop for PresenterWindow {
    fn drop(&mut self) {
        self.swapchain_helper.destroy(None).unwrap();
    }
}

impl Presenter for PresenterWindow {
    fn resize(&mut self, extents: RenderSurfaceExtents) {
        self.extents = extents;
    }

    fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
    ) {
        // FIXME: if the windows is minimized, we should not resize the RenderSurface and we should not present
        // the swapchain.
        if self.extents.width() > 1 && self.extents.height() > 1 {
            self.present(render_context, render_surface);
        }
    }
}
