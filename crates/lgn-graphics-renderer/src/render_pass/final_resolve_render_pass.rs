use lgn_graphics_api::{
    AddressMode, BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, CullMode,
    DepthState, DeviceContext, FilterType, Format, GraphicsPipelineDef, LoadOp, MipMapMode,
    PrimitiveTopology, RasterizerState, ResourceState, ResourceUsage, SampleCount, Sampler,
    SamplerDef, StencilOp, StoreOp, TextureView, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;

use crate::{
    cgen,
    components::RenderSurface,
    hl_gfx_api::HLCommandBuffer,
    resources::{PipelineHandle, PipelineManager},
    RenderContext,
};

pub struct FinalResolveRenderPass {
    pipeline_handle: PipelineHandle,
    linear_sampler: Sampler,
}

impl FinalResolveRenderPass {
    pub fn new(device_context: &DeviceContext, pipeline_manager: &PipelineManager) -> Self {
        Self {
            pipeline_handle: build_final_resolve_pso(pipeline_manager),
            linear_sampler: device_context.create_sampler(SamplerDef {
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

    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        resolve_rtv: &TextureView,
    ) {
        cmd_buffer.with_label("Final resolve", |cmd_buffer| {
        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.pipeline_handle)
            .unwrap();

        cmd_buffer.bind_pipeline(pipeline);

        render_surface
            .hdr_rt_mut()
            .transition_to(cmd_buffer, ResourceState::SHADER_RESOURCE);

        let mut descriptor_set = cgen::descriptor_set::FinalResolveDescriptorSet::default();
        descriptor_set.set_linear_texture(render_surface.hdr_rt().srv());
        descriptor_set.set_linear_sampler(&self.linear_sampler);

        let descriptor_set_handle = render_context.write_descriptor_set(
            cgen::descriptor_set::FinalResolveDescriptorSet::descriptor_set_layout(),
            descriptor_set.descriptor_refs(),
        );
        cmd_buffer.bind_descriptor_set(
            cgen::descriptor_set::FinalResolveDescriptorSet::descriptor_set_layout(),
            descriptor_set_handle,
        );

        #[rustfmt::skip]
        let vertex_data: [f32; 12] = [  0.0, 2.0, 0.0, 2.0,
                                          2.0, 0.0, 2.0, 0.0,
                                        0.0, 0.0, 0.0, 0.0 ];

        let mut transient_buffer_allocator = render_context.transient_buffer_allocator();

        let sub_allocation = transient_buffer_allocator
            .copy_data_slice(&vertex_data, ResourceUsage::AS_VERTEX_BUFFER);

        cmd_buffer.bind_vertex_buffer(0, sub_allocation.vertex_buffer_binding());

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: resolve_rtv,
                load_op: LoadOp::DontCare,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue([0.0; 4]),
            }],
            &None,
        );

        cmd_buffer.draw(3, 0);

        cmd_buffer.end_render_pass();
        });
    }
}

fn build_final_resolve_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::FinalResolvePipelineLayout::root_signature();

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

    let resterizer_state = lgn_graphics_api::RasterizerState {
        cull_mode: CullMode::Front,
        ..RasterizerState::default()
    };

    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(
            cgen::shader::final_resolve_shader::ID,
            cgen::shader::final_resolve_shader::NONE,
        ),
        move |device_context, shader| {
            device_context
                .create_graphics_pipeline(&GraphicsPipelineDef {
                    shader,
                    root_signature,
                    vertex_layout: &VertexLayout::default(),
                    blend_state: &BlendState::default_alpha_disabled(),
                    depth_state: &depth_state,
                    rasterizer_state: &resterizer_state,
                    color_formats: &[Format::B8G8R8A8_UNORM],
                    sample_count: SampleCount::SampleCount1,
                    depth_stencil_format: None,
                    primitive_topology: PrimitiveTopology::TriangleList,
                })
                .unwrap()
        },
    )
}
