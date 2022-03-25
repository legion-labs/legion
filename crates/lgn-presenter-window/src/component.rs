use lgn_codec_api::backends::EncoderConfig;
use lgn_codec_api::backends::{nvenc::NvEncEncoderWrapper, CodecHardware::Nvidia};
use lgn_codec_api::encoder_work_queue::{EncoderWorkItem, EncoderWorkQueue};
use lgn_codec_api::VideoProcessor;
use lgn_graphics_api::prelude::*;
use lgn_graphics_renderer::{
    components::{Presenter, RenderSurface, RenderSurfaceExtents},
    RenderContext, Renderer,
};
use raw_window_handle::HasRawWindowHandle;
use std::convert::TryFrom;

use crate::SwapchainHelper;

pub struct PresenterWindow {
    swapchain_helper: SwapchainHelper,
    extents: RenderSurfaceExtents,

    // TMP - test code for Cuda encoder
    cuda_encoder: Option<NvEncEncoderWrapper>,
}

impl std::fmt::Debug for PresenterWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterWindow").finish()
    }
}

impl PresenterWindow {
    pub fn from_window(
        renderer: &Renderer,
        encoder_work_queue: &EncoderWorkQueue,
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

        let encoder_cofig = EncoderConfig {
            hardware: Nvidia,
            gfx_config: device_context.clone(),
            work_queue: encoder_work_queue.clone(),
            width: extents.width(),
            height: extents.height(),
        };

        Self {
            swapchain_helper: SwapchainHelper::new(device_context, swapchain, None).unwrap(),
            extents,
            cuda_encoder: NvEncEncoderWrapper::new(encoder_cofig),
        }
    }

    pub fn present(
        &mut self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) {
        // TMP - test encoder
        if let Some(encoder) = &self.cuda_encoder {
            encoder
                .submit_input(&EncoderWorkItem {
                    image: render_surface.texture().clone(),
                    semaphore: render_surface.encoder_sem().clone(),
                })
                .unwrap();
        }

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
        let cmd_buffer = render_context.alloc_command_buffer();
        let swapchain_texture = presentable_frame.swapchain_texture();

        {
            render_surface.transition_to(&cmd_buffer, ResourceState::COPY_SRC);

            cmd_buffer.resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    swapchain_texture,
                    ResourceState::PRESENT,
                    ResourceState::COPY_DST,
                )],
            );

            let src_texture = render_surface.texture().external_resource();
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
            cmd_buffer.blit_texture(&src_texture, dst_texture, &blit_params);

            cmd_buffer.resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    swapchain_texture,
                    ResourceState::COPY_DST,
                    ResourceState::PRESENT,
                )],
            );
        }

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
