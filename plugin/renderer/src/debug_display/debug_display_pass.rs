use crate::components::RenderSurface;
use crate::{RenderContext, Renderer};
use lgn_graphics_api::prelude::*;

pub struct DebugDisplayPass {
    root_signature: RootSignature,
    pipeline: Pipeline,
}

impl DebugDisplayPass {
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        let (shader, root_signature) = renderer.prepare_vs_ps(
            String::from_utf8(include_bytes!("../../shaders/shader.hlsl").to_vec()).unwrap(),
        );
        //
        // Pipeline state
        //
        let vertex_layout = VertexLayout {
            attributes: vec![
                VertexLayoutAttribute {
                    format: Format::R32G32B32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    byte_offset: 0,
                    gl_attribute_name: Some("pos".to_owned()),
                },
                VertexLayoutAttribute {
                    format: Format::R32G32B32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    byte_offset: 12,
                    gl_attribute_name: Some("normal".to_owned()),
                },
            ],
            buffers: vec![VertexLayoutBuffer {
                stride: 24,
                rate: VertexAttributeRate::Vertex,
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
                root_signature: &root_signature,
                vertex_layout: &vertex_layout,
                blend_state: &BlendState::default_alpha_enabled(),
                depth_state: &depth_state,
                rasterizer_state: &RasterizerState {
                    fill_mode: FillMode::Wireframe,
                    ..RasterizerState::default()
                },
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        Self {
            root_signature,
            pipeline,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(
        &self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &CommandBuffer,
        render_surface: &RenderSurface,
    ) {
    }
}
