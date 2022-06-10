use lgn_graphics_api::prelude::*;
use lgn_graphics_renderer::{
    components::{Presenter, RenderSurface, RenderSurfaceExtents},
    RenderContext, Renderer,
};
use raw_window_handle::HasRawWindowHandle;

use crate::SwapchainHelper;

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
        let swapchain = device_context.create_swapchain(
            hwnd,
            SwapchainDef::new(extents.width(), extents.height(), true),
        );

        Self {
            swapchain_helper: SwapchainHelper::new(device_context, swapchain, None),
            extents,
        }
    }

    pub fn present(
        &mut self,
        render_context: &mut RenderContext<'_>,
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

        let swapchain_texture = presentable_frame.swapchain_texture();

        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        // TODO(jsg): A single viewport for now, must have a "viewport compositor" eventually.
        let final_target = render_surface.viewports()[0].view_target();

        assert_eq!(
            final_target.definition().extents,
            swapchain_texture.definition().extents
        );

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[
                TextureBarrier::state_transition(
                    swapchain_texture,
                    ResourceState::PRESENT,
                    ResourceState::COPY_DST,
                ),
                TextureBarrier::state_transition(
                    final_target,
                    ResourceState::RENDER_TARGET,
                    ResourceState::COPY_SRC,
                ),
            ],
        );

        cmd_buffer.cmd_copy_image(
            final_target,
            swapchain_texture,
            &CmdCopyTextureParams {
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::COPY_DST,
                src_offset: Offset3D { x: 0, y: 0, z: 0 },
                dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                src_mip_level: 0,
                dst_mip_level: 0,
                src_array_slice: 0,
                dst_array_slice: 0,
                src_plane_slice: PlaneSlice::Default,
                dst_plane_slice: PlaneSlice::Default,
                extent: final_target.definition().extents,
            },
        );

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[
                TextureBarrier::state_transition(
                    swapchain_texture,
                    ResourceState::COPY_DST,
                    ResourceState::PRESENT,
                ),
                TextureBarrier::state_transition(
                    final_target,
                    ResourceState::COPY_SRC,
                    ResourceState::RENDER_TARGET,
                ),
            ],
        );

        cmd_buffer.end();

        //
        // Present
        //
        {
            let wait_sem = render_surface.presenter_sem();
            presentable_frame
                .present(
                    &mut render_context.graphics_queue.queue_mut(),
                    wait_sem,
                    &[cmd_buffer],
                )
                .unwrap();
        }

        render_context
            .transient_commandbuffer_allocator
            .release(cmd_buffer_handle);
    }
}

impl Drop for PresenterWindow {
    fn drop(&mut self) {
        self.swapchain_helper.destroy(None).unwrap();
    }
}

impl Presenter for PresenterWindow {
    fn resize(&mut self, _device_context: &DeviceContext, extents: RenderSurfaceExtents) {
        self.extents = extents;
    }

    fn present(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) {
        // FIXME: if the windows is minimized, we should not resize the RenderSurface
        // and we should not present the swapchain.
        if self.extents.width() > 1 && self.extents.height() > 1 {
            self.present(render_context, render_surface);
        }
    }
}
