use lgn_graphics_api::prelude::*;

use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    resources::ShaderManager,
    RenderContext,
};
use lgn_tracing::span_fn;

use crate::{cgen, tmp_shader_data::display_mapper_shader_family};

pub struct OffscreenHelper {
    render_image: Texture,
    render_image_rtv: TextureView,
    copy_image: Texture,
    pipeline: Pipeline,
    bilinear_sampler: Sampler,
}

impl OffscreenHelper {
    pub fn new(
        shader_manager: &ShaderManager,
        device_context: &DeviceContext,
        resolution: RenderSurfaceExtents,
    ) -> anyhow::Result<Self> {
        let root_signature = cgen::pipeline_layout::DisplayMapperPipelineLayout::root_signature();

        let shader_handle = shader_manager.register_shader(CGenShaderKey::make(
            display_mapper_shader_family::ID,
            display_mapper_shader_family::NONE,
        ));

        let shader = shader_manager.get_shader(shader_handle).unwrap();

        let pipeline = device_context.create_graphics_pipeline(&GraphicsPipelineDef {
            shader,
            root_signature,
            vertex_layout: &VertexLayout::default(),
            blend_state: &BlendState::default(),
            depth_state: &DepthState::default(),
            rasterizer_state: &RasterizerState {
                cull_mode: CullMode::Back,
                ..RasterizerState::default()
            },
            primitive_topology: PrimitiveTopology::TriangleList,
            color_formats: &[Format::R8G8B8A8_UNORM],
            depth_stencil_format: None,
            sample_count: SampleCount::SampleCount1,
        })?;

        let sampler_def = SamplerDef {
            min_filter: FilterType::Linear,
            mag_filter: FilterType::Linear,
            mip_map_mode: MipMapMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            ..SamplerDef::default()
        };
        let bilinear_sampler = device_context.create_sampler(&sampler_def)?;

        let render_image = device_context.create_texture(&TextureDef {
            extents: Extents3D {
                width: resolution.width(),
                height: resolution.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            mem_usage: MemoryUsage::GpuOnly,
            usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            tiling: TextureTiling::Optimal,
        })?;

        let render_image_rtv = render_image.create_view(&TextureViewDef::as_render_target_view(
            render_image.definition(),
        ))?;

        let copy_image = device_context.create_texture(&TextureDef {
            extents: Extents3D {
                width: resolution.width(),
                height: resolution.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            mem_usage: MemoryUsage::GpuToCpu,
            usage_flags: ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            tiling: TextureTiling::Linear,
        })?;

        Ok(Self {
            render_image,
            render_image_rtv,
            copy_image,
            pipeline,
            bilinear_sampler,
        })
    }

    #[span_fn]
    pub fn present<F: FnOnce(&[u8], usize)>(
        &mut self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        copy_fn: F,
    ) -> anyhow::Result<()> {
        let mut cmd_buffer = render_context.alloc_command_buffer();
        let render_texture = &self.render_image;
        let render_texture_rtv = &self.render_image_rtv;
        let copy_texture = &self.copy_image;

        render_surface.transition_to(&cmd_buffer, ResourceState::SHADER_RESOURCE);

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                render_texture,
                ResourceState::COPY_SRC,
                ResourceState::RENDER_TARGET,
            )],
        );

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_texture_rtv,
                load_op: LoadOp::DontCare,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue::default(),
            }],
            &None,
        );

        cmd_buffer.bind_pipeline(&self.pipeline);

        let mut descriptor_set = cgen::descriptor_set::DisplayMapperDescriptorSet::default();
        descriptor_set.set_hdr_image(render_surface.shader_resource_view());
        descriptor_set.set_hdr_sampler(&self.bilinear_sampler);
        let descriptor_set_handle = render_context.write_descriptor_set(&descriptor_set);
        cmd_buffer.bind_descriptor_set_handle(descriptor_set_handle);

        cmd_buffer.draw(3, 0);

        cmd_buffer.end_render_pass();

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                render_texture,
                ResourceState::RENDER_TARGET,
                ResourceState::COPY_SRC,
            )],
        );

        //
        // Copy
        //

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                copy_texture,
                ResourceState::COMMON,
                ResourceState::COPY_DST,
            )],
        );

        let copy_extents = render_texture.definition().extents;
        assert_eq!(copy_texture.definition().extents, copy_extents);

        cmd_buffer.copy_image(
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

        cmd_buffer.resource_barrier(
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

        let wait_sem = render_surface.sema();
        let graphics_queue = render_context.graphics_queue();

        graphics_queue.submit(&mut [cmd_buffer.finalize()], &[wait_sem], &[], None);

        graphics_queue.wait_for_queue_idle()?;

        let sub_resource = copy_texture.map_texture(PlaneSlice::Default)?;
        copy_fn(sub_resource.data, sub_resource.row_pitch as usize);
        copy_texture.unmap_texture();
        Ok(())
    }
}
