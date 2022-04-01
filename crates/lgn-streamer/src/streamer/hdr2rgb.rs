use lgn_codec_api::{encoder_resource::EncoderResource, stream_encoder::StreamEncoder};
use lgn_graphics_api::{
    DeviceContext, Format, GPUViewType, ResourceState, ResourceUsage, Semaphore, SemaphoreDef,
    SemaphoreUsage, Texture, TextureBarrier,
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
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) {
        let mut cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.resource_barrier(
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
            &mut cmd_buffer,
            self.resolve_rt.rtv(),
        );

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                self.resolve_rt.texture(),
                ResourceState::RENDER_TARGET,
                ResourceState::PRESENT,
            )],
        );

        let wait_sem = render_surface.presenter_sem();
        render_context.graphics_queue().submit(
            &mut [cmd_buffer.finalize()],
            &[wait_sem],
            &[&self.export_semaphore.external_resource()],
            None,
        );
    }
}
