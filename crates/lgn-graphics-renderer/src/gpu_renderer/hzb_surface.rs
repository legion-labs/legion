use lgn_graphics_api::{
    AddressMode, BlendState, ColorClearValue, ColorRenderTargetBinding, CommandBuffer, CompareOp,
    CullMode, DepthState, DeviceContext, Extents3D, FilterType, Format, GraphicsPipelineDef,
    LoadOp, MemoryUsage, MipMapMode, PrimitiveTopology, RasterizerState, ResourceFlags,
    ResourceState, ResourceUsage, SampleCount, Sampler, SamplerDef, StencilOp, StoreOp, Texture,
    TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::{
    cgen,
    components::RenderSurfaceExtents,
    resources::{PipelineDef, PipelineHandle, PipelineManager},
    RenderContext,
};

pub(crate) struct HzbSurface {
    texture: Texture,
    srv_view: TextureView,
    srv_mip_views: Vec<TextureView>,
    rt_mip_view: Vec<TextureView>,
    pipeline_handle: PipelineHandle,
    mip_sampler: Sampler,
}

impl HzbSurface {
    pub fn new(
        device_context: &DeviceContext,
        extents: RenderSurfaceExtents,
        pipeline_manager: &PipelineManager,
    ) -> Self {
        const SCALE_THRESHOLD: f32 = 0.7;

        let mut hzb_width = 2.0f32.powf((extents.width() as f32).log2().floor());
        if hzb_width / extents.width() as f32 > SCALE_THRESHOLD {
            hzb_width /= 2.0;
        }
        let mut hzb_height = 2.0f32.powf((extents.height() as f32).log2().floor());
        if hzb_height / extents.height() as f32 > SCALE_THRESHOLD {
            hzb_height /= 2.0;
        }

        hzb_width = hzb_width.max(4.0);
        hzb_height = hzb_height.max(4.0);

        let mut min_extent = hzb_width.min(hzb_height) as u32;
        let mut mip_count = 1;
        while min_extent != 1 {
            min_extent /= 2;
            mip_count += 1;
        }

        let hzb_def = TextureDef {
            extents: Extents3D {
                width: hzb_width as u32,
                height: hzb_height as u32,
                depth: 1,
            },
            array_length: 1,
            mip_count,
            format: Format::R32_SFLOAT,
            usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_SHADER_RESOURCE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let texture = device_context.create_texture(hzb_def, "HZB");
        let srv_view = texture.create_view(TextureViewDef::as_shader_resource_view(
            texture.definition(),
        ));

        let mut srv_mip_views = Vec::with_capacity(mip_count as usize);
        let mut rt_mip_view = Vec::with_capacity(mip_count as usize);

        for mip_index in 0..mip_count {
            srv_mip_views.push(
                texture.create_view(TextureViewDef::as_srv_with_mip_spec(&hzb_def, mip_index, 1)),
            );

            rt_mip_view
                .push(texture.create_view(TextureViewDef::as_rt_for_mip(&hzb_def, mip_index)));
        }

        Self {
            texture,
            srv_view,
            srv_mip_views,
            rt_mip_view,
            pipeline_handle: build_hzb_pso(pipeline_manager),
            mip_sampler: device_context.create_sampler(SamplerDef {
                min_filter: FilterType::Nearest,
                mag_filter: FilterType::Nearest,
                mip_map_mode: MipMapMode::Nearest,
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mip_lod_bias: 0.0,
                max_anisotropy: 1.0,
                compare_op: CompareOp::Never,
            }),
        }
    }

    pub fn hzb_srv_view(&self) -> &TextureView {
        &self.srv_view
    }

    pub fn hzb_pixel_extents(&self) -> Vec2 {
        Vec2::new(
            self.texture.definition().extents.width as f32,
            self.texture.definition().extents.height as f32,
        )
    }

    pub fn hzb_max_lod(&self) -> u32 {
        self.texture.definition().mip_count - 1
    }

    pub fn generate_hzb(
        &self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        depth_srv_view: &TextureView,
    ) {
        let pipeline = render_context
            .pipeline_manager
            .get_pipeline(self.pipeline_handle)
            .unwrap();

        cmd_buffer.cmd_bind_pipeline(pipeline);

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                &self.texture,
                ResourceState::SHADER_RESOURCE,
                ResourceState::RENDER_TARGET,
            )],
        );

        for (index, rt_view) in self.rt_mip_view.iter().enumerate() {
            let mut descriptor_set = cgen::descriptor_set::HzbDescriptorSet::default();
            descriptor_set.set_depth_texture(if index == 0 {
                depth_srv_view
            } else {
                &self.srv_mip_views[index - 1]
            });

            descriptor_set.set_depth_sampler(&self.mip_sampler);

            let descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::HzbDescriptorSet::descriptor_set_layout(),
                descriptor_set.descriptor_refs(),
            );
            cmd_buffer.cmd_bind_descriptor_set_handle(
                cgen::descriptor_set::HzbDescriptorSet::descriptor_set_layout(),
                descriptor_set_handle,
            );

            cmd_buffer.cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: rt_view,
                    load_op: LoadOp::DontCare,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue([0.0; 4]),
                }],
                &None,
            );

            cmd_buffer.cmd_draw(3, 0);

            cmd_buffer.cmd_end_render_pass();

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition_for_mip(
                    &self.texture,
                    ResourceState::RENDER_TARGET,
                    ResourceState::SHADER_RESOURCE,
                    Some(index as u8),
                )],
            );
        }
    }
}

fn build_hzb_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::HzbPipelineLayout::root_signature();

    let depth_state = DepthState {
        depth_test_enable: false,
        depth_write_enable: false,
        depth_compare_op: CompareOp::Never,
        stencil_test_enable: false,
        stencil_read_mask: 0xFF,
        stencil_write_mask: 0xFF,
        front_depth_fail_op: StencilOp::default(),
        front_stencil_compare_op: CompareOp::default(),
        front_stencil_fail_op: StencilOp::default(),
        front_stencil_pass_op: StencilOp::default(),
        back_depth_fail_op: StencilOp::default(),
        back_stencil_compare_op: CompareOp::default(),
        back_stencil_fail_op: StencilOp::default(),
        back_stencil_pass_op: StencilOp::default(),
    };

    let rasterizer_state = RasterizerState {
        cull_mode: CullMode::Back,
        ..RasterizerState::default()
    };

    let shader = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(cgen::shader::hzb_shader::ID, cgen::shader::hzb_shader::NONE),
        )
        .unwrap();
    pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
        shader,
        root_signature: root_signature.clone(),
        vertex_layout: VertexLayout::default(),
        blend_state: BlendState::default_alpha_disabled(),
        depth_state,
        rasterizer_state,
        color_formats: vec![Format::R32_SFLOAT],
        sample_count: SampleCount::SampleCount1,
        depth_stencil_format: None,
        primitive_topology: PrimitiveTopology::TriangleList,
    }))
}
