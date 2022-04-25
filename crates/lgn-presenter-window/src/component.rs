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

    pub fn present(
        &mut self,
        render_context: &RenderContext<'_>,
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

        let cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                swapchain_texture,
                ResourceState::PRESENT,
                ResourceState::RENDER_TARGET,
            )],
        );

        // final resolve
        let final_resolve_render_pass = render_surface.final_resolve_render_pass();
        let final_resolve_render_pass = final_resolve_render_pass.write();

        final_resolve_render_pass.render(
            render_context,
            render_surface,
            &cmd_buffer,
            presentable_frame.swapchain_rtv(),
        );

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                swapchain_texture,
                ResourceState::RENDER_TARGET,
                ResourceState::PRESENT,
            )],
        );

        //
        // Present
        //
        {
            let present_queue = render_context.graphics_queue();
            let wait_sem = render_surface.presenter_sem();
            presentable_frame
                .present(&present_queue, wait_sem, &mut [cmd_buffer.finalize()])
                .unwrap();
        }
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

    fn present(&mut self, render_context: &RenderContext<'_>, render_surface: &mut RenderSurface) {
        // FIXME: if the windows is minimized, we should not resize the RenderSurface
        // and we should not present the swapchain.
        if self.extents.width() > 1 && self.extents.height() > 1 {
            self.present(render_context, render_surface);
        }
    }
}
