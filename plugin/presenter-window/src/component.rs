use std::cmp::max;

use graphics_api::{
    CmdBlitParams, CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, DefaultApi,
    DeviceContext, Extents2D, FilterType, GfxApi, Offset3D, Queue, ResourceState, SwapchainDef,
    Texture, TextureBarrier,
};
use legion_ecs::prelude::Component;
use legion_renderer::{
    components::{RenderSurface, RenderSurfaceId},
    Renderer,
};
use legion_window::{Window, WindowId};
use raw_window_handle::HasRawWindowHandle;

use crate::swapchain_helper::SwapchainHelper;

#[derive(Component)]
pub struct PresenterWindow {
    window_id: WindowId,
    render_surface_id: RenderSurfaceId,
    swapchain_helper: SwapchainHelper<DefaultApi>,
    cmd_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    cmd_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,
}

impl std::fmt::Debug for PresenterWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterWindow").finish()
    }
}

impl PresenterWindow {
    pub fn from_window(
        renderer: &Renderer,
        wnd: &Window,
        hwnd: &dyn HasRawWindowHandle,
        render_surface_id: RenderSurfaceId,
    ) -> Self {
        let extents = Self::get_window_extents(wnd);
        let device_context = renderer.device_context();
        let present_queue = renderer.graphics_queue();
        let swapchain = device_context
            .create_swapchain(
                hwnd,
                &SwapchainDef {
                    width: extents.width,
                    height: extents.height,
                    enable_vsync: true,
                },
            )
            .unwrap();

        let swapchain_helper =
            SwapchainHelper::<DefaultApi>::new(device_context, swapchain, None).unwrap();

        let mut cmd_pools = Vec::with_capacity(swapchain_helper.image_count());
        let mut cmd_buffers = Vec::with_capacity(swapchain_helper.image_count());
        for _ in 0..swapchain_helper.image_count() {
            let cmd_pool = present_queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap();

            let cmd_buffer = cmd_pool
                .create_command_buffer(&CommandBufferDef {
                    is_secondary: false,
                })
                .unwrap();

            cmd_pools.push(cmd_pool);
            cmd_buffers.push(cmd_buffer);
        }

        Self {
            window_id: wnd.id(),
            render_surface_id,
            swapchain_helper,
            cmd_pools,
            cmd_buffers,
        }
    }

    pub fn present(
        &mut self,
        wnd: &Window,
        present_queue: &<DefaultApi as GfxApi>::Queue,
        wait_sem: &<DefaultApi as GfxApi>::Semaphore,
        render_surface: Option<&mut RenderSurface>,
    ) {
        //
        // Acquire backbuffer
        //
        let extents = Self::get_window_extents(wnd);
        let presentable_frame = self
            .swapchain_helper
            .acquire_next_image(extents.width, extents.height, None)
            .unwrap();

        //
        // Blit render surface
        //
        let cmd_buffer = {
            let swapchain_texture = presentable_frame.swapchain_texture();
            let cmd_pool = &self.cmd_pools[presentable_frame.rotating_frame_index()];
            let cmd_buffer = &self.cmd_buffers[presentable_frame.rotating_frame_index()];

            cmd_pool.reset_command_pool().unwrap();
            cmd_buffer.begin().unwrap();

            if let Some(render_surface) = render_surface {
                render_surface.transition_to(cmd_buffer, ResourceState::COPY_SRC);

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
                let src_texture_def = src_texture.texture_def();
                let dst_texture = swapchain_texture;
                let dst_texture_def = dst_texture.texture_def();
                let blit_params = CmdBlitParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offsets: [
                        Offset3D { x: 0, y: 0, z: 0 },
                        Offset3D {
                            x: src_texture_def.extents.width as i32,
                            y: src_texture_def.extents.height as i32,
                            z: 1,
                        },
                    ],
                    dst_offsets: [
                        Offset3D { x: 0, y: 0, z: 0 },
                        Offset3D {
                            x: dst_texture_def.extents.width as i32,
                            y: dst_texture_def.extents.height as i32,
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

            cmd_buffer
        };

        //
        // Present
        //
        presentable_frame
            .present(&present_queue, wait_sem, &[cmd_buffer])
            .unwrap();
    }

    pub fn window_id(&self) -> WindowId {
        self.window_id
    }

    pub fn render_surface_id(&self) -> RenderSurfaceId {
        self.render_surface_id
    }

    fn get_window_extents(wnd: &Window) -> Extents2D {
        Extents2D {
            width: max(1u32, wnd.physical_width()),
            height: max(1u32, wnd.physical_height()),
        }
    }
}

impl Drop for PresenterWindow {
    fn drop(&mut self) {
        self.swapchain_helper.destroy(None).unwrap();
    }
}
