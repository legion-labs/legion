use lgn_graphics_api::prelude::*;

use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_graphics_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    resources::{PipelineDef, PipelineHandle, PipelineManager},
    RenderContext,
};
use lgn_tracing::span_fn;

use crate::cgen;

pub struct OffscreenHelper {
    render_image: Texture,
    render_image_rtv: TextureView,
    copy_image: Texture,
    pipeline_handle: PipelineHandle,
    bilinear_sampler: Sampler,
}

impl OffscreenHelper {
    pub fn new(
        pipeline_manager: &PipelineManager,
        device_context: &DeviceContext,
        resolution: RenderSurfaceExtents,
    ) -> Self {
        let root_signature = cgen::pipeline_layout::DisplayMapperPipelineLayout::root_signature();

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::display_mapper_shader::ID,
                    cgen::shader::display_mapper_shader::NONE,
                ),
            )
            .unwrap();
        let pipeline_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader,
                root_signature: root_signature.clone(),
                vertex_layout: VertexLayout::default(),
                blend_state: BlendState::default(),
                depth_state: DepthState::default(),
                rasterizer_state: RasterizerState {
                    cull_mode: CullMode::Back,
                    ..RasterizerState::default()
                },
                primitive_topology: PrimitiveTopology::TriangleList,
                color_formats: vec![Format::R8G8B8A8_UNORM],
                depth_stencil_format: None,
                sample_count: SampleCount::SampleCount1,
            }));

        let bilinear_sampler = device_context.create_sampler(SamplerDef {
            min_filter: FilterType::Linear,
            mag_filter: FilterType::Linear,
            mip_map_mode: MipMapMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            ..SamplerDef::default()
        });

        let render_image = device_context.create_texture(
            TextureDef {
                extents: Extents3D {
                    width: resolution.width(),
                    height: resolution.height(),
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
                memory_usage: MemoryUsage::GpuOnly,
                usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Optimal,
            },
            "Offscreen",
        );

        let render_image_rtv = render_image.create_view(TextureViewDef::as_render_view(
            render_image.definition(),
            GPUViewType::RenderTarget,
        ));

        let copy_image = device_context.create_texture(
            TextureDef {
                extents: Extents3D {
                    width: resolution.width(),
                    height: resolution.height(),
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
                memory_usage: MemoryUsage::GpuToCpu,
                usage_flags: ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Linear,
            },
            "Copy",
        );

        Self {
            render_image,
            render_image_rtv,
            copy_image,
            pipeline_handle,
            bilinear_sampler,
        }
    }

    #[span_fn]
    pub fn present<F: FnOnce(&[u8], usize)>(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
        copy_fn: F,
    ) -> anyhow::Result<()> {
        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        let render_texture = &self.render_image;
        let render_texture_rtv = &self.render_image_rtv;
        let copy_texture = &self.copy_image;

        render_surface.composite_viewports(cmd_buffer);
        let final_target_srv = render_surface.final_target_srv();

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                render_texture,
                ResourceState::COPY_SRC,
                ResourceState::RENDER_TARGET,
            )],
        );

        cmd_buffer.cmd_begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_texture_rtv,
                load_op: LoadOp::DontCare,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue::default(),
            }],
            &None,
        );

        let pipeline = render_context
            .pipeline_manager
            .get_pipeline(self.pipeline_handle)
            .unwrap();
        cmd_buffer.cmd_bind_pipeline(pipeline);

        let mut descriptor_set = cgen::descriptor_set::DisplayMapperDescriptorSet::default();
        descriptor_set.set_hdr_image(final_target_srv);
        descriptor_set.set_hdr_sampler(&self.bilinear_sampler);
        let descriptor_set_handle = render_context.write_descriptor_set(
            cgen::descriptor_set::DisplayMapperDescriptorSet::descriptor_set_layout(),
            descriptor_set.descriptor_refs(),
        );
        cmd_buffer.cmd_bind_descriptor_set_handle(
            cgen::descriptor_set::DisplayMapperDescriptorSet::descriptor_set_layout(),
            descriptor_set_handle,
        );

        cmd_buffer.cmd_draw(3, 0);

        cmd_buffer.cmd_end_render_pass();

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[
                TextureBarrier::state_transition(
                    render_texture,
                    ResourceState::RENDER_TARGET,
                    ResourceState::COPY_SRC,
                ),
                TextureBarrier::state_transition(
                    copy_texture,
                    ResourceState::COMMON,
                    ResourceState::COPY_DST,
                ),
            ],
        );

        //
        // Copy
        //

        let copy_extents = render_texture.definition().extents;
        assert_eq!(copy_texture.definition().extents, copy_extents);

        cmd_buffer.cmd_copy_image(
            render_texture,
            copy_texture,
            &CmdCopyTextureParams {
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::COPY_DST,
                src_offset: Offset3D { x: 0, y: 0, z: 0 },
                dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                src_mip_level: 0,
                dst_mip_level: 0,
                src_array_slice: 0,
                dst_array_slice: 0,
                extent: copy_extents,
                src_plane_slice: PlaneSlice::Default,
                dst_plane_slice: PlaneSlice::Default,
            },
        );

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                copy_texture,
                ResourceState::COPY_DST,
                ResourceState::COMMON,
            )],
        );

        //
        // Present the image
        //

        cmd_buffer.end();

        let wait_sem = render_surface.presenter_sem();

        render_context
            .graphics_queue
            .queue_mut()
            .submit(&[cmd_buffer], &[wait_sem], &[], None);

        render_context
            .graphics_queue
            .queue_mut()
            .wait_for_queue_idle();

        let sub_resource = copy_texture.map_texture(PlaneSlice::Default)?;
        copy_fn(sub_resource.data, sub_resource.row_pitch as usize);
        copy_texture.unmap_texture();

        render_context
            .transient_commandbuffer_allocator
            .release(cmd_buffer_handle);

        Ok(())
    }
}
