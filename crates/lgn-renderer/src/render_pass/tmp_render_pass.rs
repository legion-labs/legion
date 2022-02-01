#![allow(unsafe_code)]

use lgn_embedded_fs::embedded_watched_file;
use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DeviceContext, Format,
    GraphicsPipelineDef, LoadOp, Pipeline, PrimitiveTopology, RasterizerState, ResourceState,
    SampleCount, StencilOp, StoreOp, VertexAttributeRate, VertexLayout, VertexLayoutAttribute,
    VertexLayoutBuffer,
};
use lgn_tracing::span_fn;

use crate::{
    cgen,
    components::{RenderSurface, StaticMesh},
    hl_gfx_api::{HLCommandBuffer, ShaderManager},
    RenderContext,
};

pub struct TmpRenderPass {
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

embedded_watched_file!(INCLUDE_BRDF, "gpu/include/brdf.hsh");
embedded_watched_file!(SHADER_SHADER, "gpu/shaders/shader.hlsl");

impl TmpRenderPass {
    pub fn new(device_context: &DeviceContext, shader_manager: &ShaderManager) -> Self {
        let root_signature = cgen::pipeline_layout::ShaderPipelineLayout::root_signature();

        let shader = shader_manager.prepare_vs_ps(SHADER_SHADER.path());

        //
        // Pipeline state
        //
        let vertex_layout = VertexLayout {
            attributes: vec![VertexLayoutAttribute {
                format: Format::R32_UINT,
                buffer_index: 0,
                location: 0,
                byte_offset: 0,
            }],
            buffers: vec![VertexLayoutBuffer {
                stride: 4,
                rate: VertexAttributeRate::Instance,
            }],
        };

        let depth_state = DepthState {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_op: CompareOp::Less,
            stencil_test_enable: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            front_depth_fail_op: StencilOp::default(),
            front_stencil_compare_op: CompareOp::Always,
            front_stencil_fail_op: StencilOp::default(),
            front_stencil_pass_op: StencilOp::default(),
            back_depth_fail_op: StencilOp::default(),
            back_stencil_compare_op: CompareOp::Always,
            back_stencil_fail_op: StencilOp::default(),
            back_stencil_pass_op: StencilOp::default(),
        };

        let pipeline = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
                root_signature,
                vertex_layout: &vertex_layout,
                blend_state: &BlendState::default_alpha_enabled(),
                depth_state: &depth_state,
                rasterizer_state: &RasterizerState::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        Self {
            pipeline,
            color: [0f32, 0f32, 0.2f32, 1.0f32],
            speed: 1.0f32,
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    #[span_fn]
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[&StaticMesh],
    ) {
        render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_surface.render_target_view(),
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue([0.2, 0.2, 0.2, 1.0]),
            }],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_stencil_texture_view(),
                depth_load_op: LoadOp::Clear,
                stencil_load_op: LoadOp::DontCare,
                depth_store_op: StoreOp::Store,
                stencil_store_op: StoreOp::DontCare,
                clear_value: DepthStencilClearValue {
                    depth: 1.0,
                    stencil: 0,
                },
            }),
        );

        cmd_buffer.bind_pipeline(&self.pipeline);
        cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

        for (_index, static_mesh) in static_meshes.iter().enumerate() {
            cmd_buffer.draw_instanced(static_mesh.num_vertices, 0, 1, static_mesh.gpu_instance_id);
        }

        cmd_buffer.end_render_pass();
    }
}
