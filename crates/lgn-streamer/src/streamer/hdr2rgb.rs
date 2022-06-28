use lgn_codec_api::{encoder_resource::EncoderResource, stream_encoder::StreamEncoder};
use lgn_graphics_api::{
    CmdCopyTextureParams, DeviceContext, Extents3D, Format, MemoryUsage, Offset3D, PlaneSlice,
    ResourceFlags, ResourceState, ResourceUsage, Semaphore, SemaphoreDef, SemaphoreUsage, Texture,
    TextureBarrier, TextureDef, TextureTiling,
};
use lgn_graphics_renderer::{components::RenderSurface, RenderContext};

use super::Resolution;

pub(crate) struct Hdr2Rgb {
    resolve_rt: Texture,
    export_texture: EncoderResource<Texture>,
    export_semaphore: EncoderResource<Semaphore>,
    stream_encoder: StreamEncoder,
    counter: u64,
}

impl Hdr2Rgb {
    pub(crate) fn new(
        device_context: &DeviceContext,
        stream_encoder: &StreamEncoder,
        resolution: Resolution,
    ) -> Self {
        let resolve_rt = device_context.create_texture(
            TextureDef {
                extents: Extents3D {
                    width: resolution.width,
                    height: resolution.height,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::B8G8R8A8_UNORM,
                memory_usage: MemoryUsage::GpuOnly,
                usage_flags: ResourceUsage::AS_RENDER_TARGET
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_TRANSFERABLE
                    | ResourceUsage::AS_EXPORT_CAPABLE,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Optimal,
            },
            "Resolve_RT",
        );
        let export_texture = stream_encoder.new_external_image(&resolve_rt, device_context);

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
            counter: 0,
        }
    }

    pub fn resize(&mut self, device_context: &DeviceContext, resolution: Resolution) -> bool {
        let extents = self.resolve_rt.definition().extents;
        if extents.width != resolution.width || extents.height != resolution.height {
            self.resolve_rt = device_context.create_texture(
                TextureDef {
                    extents: Extents3D {
                        width: resolution.width,
                        height: resolution.height,
                        depth: 1,
                    },
                    array_length: 1,
                    mip_count: 1,
                    format: Format::B8G8R8A8_UNORM,
                    memory_usage: MemoryUsage::GpuOnly,
                    usage_flags: ResourceUsage::AS_RENDER_TARGET
                        | ResourceUsage::AS_SHADER_RESOURCE
                        | ResourceUsage::AS_TRANSFERABLE
                        | ResourceUsage::AS_EXPORT_CAPABLE,
                    resource_flags: ResourceFlags::empty(),
                    tiling: TextureTiling::Optimal,
                },
                "Resolve_RT",
            );
            self.export_texture = self
                .stream_encoder
                .new_external_image(&self.resolve_rt, device_context);
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

    pub(crate) fn export_semaphore_value(&self) -> u64 {
        self.counter
    }

    pub fn present(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) {
        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        let final_target = render_surface.final_target();

        assert_eq!(
            final_target.definition().extents,
            self.resolve_rt.definition().extents
        );

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[
                TextureBarrier::state_transition(
                    &self.resolve_rt,
                    ResourceState::PRESENT,
                    ResourceState::COPY_DST,
                ),
                TextureBarrier::state_transition(
                    final_target,
                    ResourceState::SHADER_RESOURCE,
                    ResourceState::COPY_SRC,
                ),
            ],
        );

        cmd_buffer.cmd_copy_image(
            final_target,
            &self.resolve_rt,
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
                    &self.resolve_rt,
                    ResourceState::COPY_DST,
                    ResourceState::PRESENT,
                ),
                TextureBarrier::state_transition(
                    final_target,
                    ResourceState::COPY_SRC,
                    ResourceState::SHADER_RESOURCE,
                ),
            ],
        );

        cmd_buffer.end();

        let wait_sem = render_surface.presenter_sem();

        let export_semaphore = self.export_semaphore.external_resource();

        self.counter += 1;
        export_semaphore.set_next_timeline_value(self.counter);

        render_context.graphics_queue.queue_mut().submit(
            &[cmd_buffer],
            &[wait_sem],
            &[&export_semaphore],
            None,
        );

        render_context
            .transient_commandbuffer_allocator
            .release(cmd_buffer_handle);
    }
}
