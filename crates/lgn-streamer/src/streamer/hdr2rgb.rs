use lgn_codec_api::{encoder_resource::EncoderResource, stream_encoder::StreamEncoder};
use lgn_graphics_api::{
    CmdCopyTextureParams, DeviceContext, Format, GPUViewType, Offset3D, PlaneSlice, ResourceState,
    ResourceUsage, Semaphore, SemaphoreDef, SemaphoreUsage, Texture, TextureBarrier,
};
use lgn_graphics_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    render_pass::RenderTarget,
    RenderContext,
};

use super::Resolution;

pub(crate) struct Hdr2Rgb {
    resolve_rt: RenderTarget,
    export_texture: EncoderResource<Texture>,
    export_semaphore: EncoderResource<Semaphore>,
    stream_encoder: StreamEncoder,
}

impl Hdr2Rgb {
    pub(crate) fn new(
        device_context: &DeviceContext,
        stream_encoder: &StreamEncoder,
        resolution: Resolution,
    ) -> Self {
        let resolve_rt = RenderTarget::new(
            device_context,
            "Resolve_RT",
            RenderSurfaceExtents::new(resolution.width, resolution.height),
            Format::B8G8R8A8_UNORM,
            ResourceUsage::AS_RENDER_TARGET
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_EXPORT_CAPABLE,
            GPUViewType::RenderTarget,
        );
        let export_texture =
            stream_encoder.new_external_image(resolve_rt.texture(), device_context);

        Self {
            resolve_rt,
            export_texture,
            export_semaphore: stream_encoder.new_external_semaphore(
                device_context,
                SemaphoreDef {
                    usage_flags: SemaphoreUsage::TIMELINE,
                    initial_value: 0,
                },
            ),
            stream_encoder: stream_encoder.clone(),
        }
    }

    pub fn resize(&mut self, device_context: &DeviceContext, resolution: Resolution) -> bool {
        let extents = self.resolve_rt.texture().definition().extents;
        if extents.width != resolution.width || extents.height != resolution.height {
            self.resolve_rt = RenderTarget::new(
                device_context,
                "Resolve_RT",
                RenderSurfaceExtents::new(resolution.width, resolution.height),
                Format::B8G8R8A8_UNORM,
                ResourceUsage::AS_RENDER_TARGET
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_TRANSFERABLE
                    | ResourceUsage::AS_EXPORT_CAPABLE,
                GPUViewType::RenderTarget,
            );
            self.export_texture = self
                .stream_encoder
                .new_external_image(self.resolve_rt.texture(), device_context);
            true
        } else {
            false
        }
    }

    pub(crate) fn export_texture(&self) -> EncoderResource<Texture> {
        self.export_texture.clone()
    }

    pub(crate) fn export_semaphore(&self) -> EncoderResource<Semaphore> {
        self.export_semaphore.clone()
    }

    pub fn present(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) {
        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        if render_surface.use_view_target() {
            // This means we rendered using the render graph, so the final resolve render pass
            // is already done. We just need to copy the result from the view_target to the
            // swapchain texture.
            let final_target = render_surface.view_target();

            assert_eq!(
                final_target.definition().extents,
                self.resolve_rt.texture().definition().extents
            );

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[
                    TextureBarrier::state_transition(
                        self.resolve_rt.texture(),
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
                self.resolve_rt.texture(),
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
                        self.resolve_rt.texture(),
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
        } else {
            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    self.resolve_rt.texture(),
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
                cmd_buffer,
                self.resolve_rt.rtv(),
            );

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    self.resolve_rt.texture(),
                    ResourceState::RENDER_TARGET,
                    ResourceState::PRESENT,
                )],
            );
        }

        cmd_buffer.end();

        let wait_sem = render_surface.presenter_sem();
        render_context.graphics_queue.queue_mut().submit(
            &[cmd_buffer],
            &[wait_sem],
            &[&self.export_semaphore.external_resource()],
            None,
        );

        render_context
            .transient_commandbuffer_allocator
            .release(cmd_buffer_handle);
    }
}
