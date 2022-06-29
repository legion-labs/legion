use lgn_graphics_api::{
    BlendState, CommandBuffer, CompareOp, DepthState, FillMode, Format, GraphicsPipelineDef,
    PrimitiveTopology, RasterizerState, SampleCount, StencilOp, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_graphics_data::Color;
use lgn_math::Vec4;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen::{self, cgen_type::TransformData},
    core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId},
    debug_display::{DebugDisplay, DebugPrimitiveMaterial, DebugPrimitiveType},
    resources::{MeshManager, PipelineDef, PipelineHandle, PipelineManager, RenderMesh},
    RenderContext,
};

pub struct DebugPass;

impl DebugPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        radiance_write_rt_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        let (
            solid_pso_depth_handle,
            wire_pso_depth_handle,
            solid_pso_no_depth_handle,
            wire_pso_no_depth_handle,
        ) = Self::build_pso_handles(builder.pipeline_manager);

        builder.add_graphics_pass("Debug", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, radiance_write_rt_view_id, RenderGraphLoadState::Load)
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(move |_, execute_context, cmd_buffer| {
                    let render_context = &execute_context.render_context;

                    let mesh_manager = execute_context.render_resources.get::<MeshManager>();

                    cmd_buffer
                        .cmd_bind_index_buffer(render_context.static_buffer.index_buffer_binding());

                    Self::render_debug_display(
                        render_context,
                        cmd_buffer,
                        render_context.debug_display,
                        &mesh_manager,
                        wire_pso_depth_handle,
                        solid_pso_depth_handle,
                        wire_pso_no_depth_handle,
                        solid_pso_no_depth_handle,
                    );
                })
        })
    }

    fn build_pso_handles(
        pipeline_manager: &PipelineManager,
    ) -> (
        PipelineHandle,
        PipelineHandle,
        PipelineHandle,
        PipelineHandle,
    ) {
        let root_signature = cgen::pipeline_layout::ConstColorPipelineLayout::root_signature();

        let vertex_layout = VertexLayout::default();

        let depth_state_enabled = DepthState {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_op: CompareOp::GreaterOrEqual,
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

        let depth_state_disabled = DepthState {
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: CompareOp::Greater,
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

        let wire_frame_state = RasterizerState {
            fill_mode: FillMode::Wireframe,
            ..RasterizerState::default()
        };

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::const_color_shader::ID,
                    cgen::shader::const_color_shader::NONE,
                ),
            )
            .unwrap();
        let solid_pso_depth_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader: shader.clone(),
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_alpha_enabled(),
                depth_state: depth_state_enabled,
                rasterizer_state: RasterizerState::default(),
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            }));

        let wire_pso_depth_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader: shader.clone(),
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_alpha_enabled(),
                depth_state: depth_state_enabled,
                rasterizer_state: wire_frame_state,
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::LineList,
            }));

        let solid_pso_no_depth_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader: shader.clone(),
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_alpha_enabled(),
                depth_state: depth_state_disabled,
                rasterizer_state: RasterizerState::default(),
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            }));

        let wire_pso_no_depth_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader,
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_alpha_enabled(),
                depth_state: depth_state_disabled,
                rasterizer_state: wire_frame_state,
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::LineList,
            }));

        (
            solid_pso_depth_handle,
            wire_pso_depth_handle,
            solid_pso_no_depth_handle,
            wire_pso_no_depth_handle,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_debug_display(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        debug_display: &DebugDisplay,
        mesh_manager: &MeshManager,
        wire_pso_depth_handle: PipelineHandle,
        solid_pso_depth_handle: PipelineHandle,
        wire_pso_no_depth_handle: PipelineHandle,
        solid_pso_no_depth_handle: PipelineHandle,
    ) {
        cmd_buffer.with_label("Debug_Display", |cmd_buffer| {
            // TODO(jsg) sort by material
            debug_display.render_primitives(|primitive| {
                let pso_handle = match primitive.material {
                    DebugPrimitiveMaterial::WireDepth => wire_pso_depth_handle,
                    DebugPrimitiveMaterial::SolidDepth => solid_pso_depth_handle,
                    DebugPrimitiveMaterial::WireNoDepth => wire_pso_no_depth_handle,
                    DebugPrimitiveMaterial::SolidNoDepth => solid_pso_no_depth_handle,
                };

                if let Some(pipeline) = render_context.pipeline_manager.get_pipeline(pso_handle) {
                    cmd_buffer.cmd_bind_pipeline(pipeline);
                    render_context.bind_default_descriptor_sets(cmd_buffer);

                    let mesh_reader = mesh_manager.read();

                    match primitive.primitive_type {
                        DebugPrimitiveType::DefaultMesh { default_mesh_type } => {
                            render_mesh(
                                mesh_reader.get_default_mesh(default_mesh_type),
                                &primitive.transform,
                                primitive.color,
                                1.0,
                                cmd_buffer,
                            );
                        }
                        DebugPrimitiveType::Mesh { mesh_id } => {
                            render_mesh(
                                mesh_reader.get_render_mesh(mesh_id),
                                &primitive.transform,
                                primitive.color,
                                1.0,
                                cmd_buffer,
                            );
                        }
                    };
                }
            });
        });
    }
}

fn render_mesh(
    mesh: &RenderMesh,
    world_xform: &GlobalTransform,
    color: Color,
    color_blend: f32,
    cmd_buffer: &mut CommandBuffer,
) {
    let mut push_constant_data = cgen::cgen_type::ConstColorPushConstantData::default();

    let mut transform = TransformData::default();
    transform.set_translation(world_xform.translation.into());
    transform.set_rotation(Vec4::from(world_xform.rotation).into());
    transform.set_scale(world_xform.scale.into());

    push_constant_data.set_transform(transform);
    push_constant_data.set_color(u32::from(color).into());
    push_constant_data.set_color_blend(color_blend.into());
    push_constant_data.set_mesh_description_offset(mesh.mesh_description_offset.into());

    cmd_buffer.cmd_push_constant_typed(&push_constant_data);

    cmd_buffer.cmd_draw_indexed(mesh.index_count.get(), mesh.index_offset, 0);
}
