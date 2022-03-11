use lgn_graphics_api::{
    AddressMode, BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, CullMode,
    DepthState, DeviceContext, Extents3D, FilterType, Format, GraphicsPipelineDef, LoadOp,
    MemoryUsage, MipMapMode, PrimitiveTopology, RasterizerState, ResourceFlags, ResourceState,
    ResourceUsage, SampleCount, Sampler, SamplerDef, StencilOp, StoreOp, Texture, TextureBarrier,
    TextureDef, TextureTiling, TextureView, TextureViewDef, VertexAttributeRate, VertexLayout,
    VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::{
    cgen,
    components::RenderSurfaceExtents,
    hl_gfx_api::HLCommandBuffer,
    resources::{PipelineHandle, PipelineManager},
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
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let texture = device_context.create_texture(&hzb_def);
        let srv_view_def = TextureViewDef::as_shader_resource_view(&hzb_def);
        let srv_view = texture.create_view(&srv_view_def);

        let mut srv_mip_views = Vec::with_capacity(mip_count as usize);
        let mut rt_mip_view = Vec::with_capacity(mip_count as usize);

        for mip_index in 0..mip_count {
            let hzb_srv_view_mip_def = TextureViewDef::as_srv_with_mip_spec(&hzb_def, mip_index, 1);
            srv_mip_views.push(texture.create_view(&hzb_srv_view_mip_def));

            let hzb_rt_view_mip_def = TextureViewDef::as_rt_for_mip(&hzb_def, mip_index);
            rt_mip_view.push(texture.create_view(&hzb_rt_view_mip_def));
        }

        let mip_sampler_def = SamplerDef {
            min_filter: FilterType::Nearest,
            mag_filter: FilterType::Nearest,
            mip_map_mode: MipMapMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            compare_op: CompareOp::Never,
        };

        Self {
            texture,
            srv_view,
            srv_mip_views,
            rt_mip_view,
            pipeline_handle: build_hzb_pso(pipeline_manager),
            mip_sampler: device_context.create_sampler(&mip_sampler_def),
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

    pub fn generate_hzb(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        depth_srv_view: &TextureView,
    ) {
        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.pipeline_handle)
            .unwrap();

        cmd_buffer.bind_pipeline(pipeline);

        cmd_buffer.resource_barrier(
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
            cmd_buffer.bind_descriptor_set(
                cgen::descriptor_set::HzbDescriptorSet::descriptor_set_layout(),
                descriptor_set_handle,
            );

            #[rustfmt::skip]
            let vertex_data: [f32; 12] = [0.0, 2.0, 0.0, 2.0,
                                          0.0, 0.0, 0.0, 0.0,
                                          2.0, 0.0, 2.0, 0.0];

            let sub_allocation = render_context
                .transient_buffer_allocator()
                .copy_data_slice(&vertex_data, ResourceUsage::AS_VERTEX_BUFFER);

            cmd_buffer.bind_buffer_suballocation_as_vertex_buffer(0, &sub_allocation);

            cmd_buffer.begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: rt_view,
                    load_op: LoadOp::DontCare,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue([0.0; 4]),
                }],
                &None,
            );

            cmd_buffer.draw(3, 0);

            cmd_buffer.end_render_pass();

            cmd_buffer.resource_barrier(
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

    let mut vertex_layout = VertexLayout::default();
    vertex_layout.attributes[0] = Some(VertexLayoutAttribute {
        format: Format::R32G32_SFLOAT,
        buffer_index: 0,
        location: 0,
        byte_offset: 0,
    });
    vertex_layout.attributes[1] = Some(VertexLayoutAttribute {
        format: Format::R32G32_SFLOAT,
        buffer_index: 0,
        location: 1,
        byte_offset: 8,
    });
    vertex_layout.buffers[0] = Some(VertexLayoutBuffer {
        stride: 16,
        rate: VertexAttributeRate::Vertex,
    });

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

    let resterizer_state = RasterizerState {
        cull_mode: CullMode::Front,
        ..RasterizerState::default()
    };

    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(cgen::shader::hzb_shader::ID, cgen::shader::hzb_shader::NONE),
        move |device_context, shader| {
            device_context
                .create_graphics_pipeline(&GraphicsPipelineDef {
                    shader,
                    root_signature,
                    vertex_layout: &vertex_layout,
                    blend_state: &BlendState::default_alpha_disabled(),
                    depth_state: &depth_state,
                    rasterizer_state: &resterizer_state,
                    color_formats: &[Format::R32_SFLOAT],
                    sample_count: SampleCount::SampleCount1,
                    depth_stencil_format: None,
                    primitive_topology: PrimitiveTopology::TriangleList,
                })
                .unwrap()
        },
    )
}
