use lgn_graphics_api::{
    BlendState, CommandBuffer, CompareOp, DepthState, FillMode, Format, GraphicsPipelineDef,
    PrimitiveTopology, RasterizerState, SampleCount, StencilOp, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_graphics_data::Color;
use lgn_math::{Vec3, Vec4};
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen::{self, cgen_type::TransformData},
    components::{ManipulatorComponent, VisualComponent},
    core::{
        RenderCamera, RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId, RenderViewport,
    },
    debug_display::{DebugDisplay, DebugPrimitiveType},
    picking::ManipulatorManager,
    resources::{
        DefaultMeshType, MeshManager, MeshMetaData, ModelManager, PipelineDef, PipelineHandle,
        PipelineManager,
    },
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
            _wire_pso_no_depth_handle,
        ) = Self::build_pso_handles(builder.pipeline_manager);

        builder.add_graphics_pass("Debug", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, radiance_write_rt_view_id, RenderGraphLoadState::Load)
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(move |_, execute_context, cmd_buffer| {
                    let render_context = &execute_context.render_context;

                    let mesh_manager = execute_context.render_resources.get::<MeshManager>();
                    let model_manager = execute_context.render_resources.get::<ModelManager>();

                    cmd_buffer
                        .cmd_bind_index_buffer(render_context.static_buffer.index_buffer_binding());

                    Self::render_ground_plane(
                        render_context,
                        cmd_buffer,
                        &mesh_manager,
                        wire_pso_depth_handle,
                    );

                    Self::render_picked(
                        render_context,
                        cmd_buffer,
                        render_context.picked_drawables,
                        &mesh_manager,
                        &model_manager,
                        wire_pso_depth_handle,
                        solid_pso_depth_handle,
                    );

                    Self::render_debug_display(
                        render_context,
                        cmd_buffer,
                        render_context.debug_display,
                        &mesh_manager,
                        wire_pso_depth_handle,
                    );

                    Self::render_manipulators(
                        render_context,
                        cmd_buffer,
                        render_context.manipulator_drawables,
                        &mesh_manager,
                        execute_context.debug_stuff.render_viewport,
                        execute_context.debug_stuff.render_camera,
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

    pub fn render_ground_plane(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        mesh_manager: &MeshManager,
        wire_pso_depth_handle: PipelineHandle,
    ) {
        cmd_buffer.with_label("Ground Plane", |cmd_buffer| {
            if let Some(wire_pso_depth_pipeline) = render_context
                .pipeline_manager
                .get_pipeline(wire_pso_depth_handle)
            {
                cmd_buffer.cmd_bind_pipeline(wire_pso_depth_pipeline);

                render_context.bind_default_descriptor_sets(cmd_buffer);

                render_mesh(
                    mesh_manager.get_default_mesh(DefaultMeshType::GroundPlane),
                    &GlobalTransform::identity(),
                    Color::BLACK,
                    0.0,
                    cmd_buffer,
                );
            }
        });
    }

    pub fn render_picked(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        picked_meshes: &[(&VisualComponent, &GlobalTransform)],
        mesh_manager: &MeshManager,
        model_manager: &ModelManager,
        wire_pso_depth_handle: PipelineHandle,
        solid_pso_depth_handle: PipelineHandle,
    ) {
        cmd_buffer.with_label("Picked", |cmd_buffer| {
            if let Some(wire_pso_depth_pipeline) = render_context
                .pipeline_manager
                .get_pipeline(wire_pso_depth_handle)
            {
                if let Some(solid_pso_depth_pipeline) = render_context
                    .pipeline_manager
                    .get_pipeline(solid_pso_depth_handle)
                {
                    render_context.bind_default_descriptor_sets(cmd_buffer);

                    let wireframe_cube =
                        mesh_manager.get_default_mesh(DefaultMeshType::WireframeCube);
                    for (visual_component, transform) in picked_meshes.iter() {
                        if let Some(model_resource_id) = visual_component.model_resource_id() {
                            if let Some(model) =
                                model_manager.get_model_meta_data(model_resource_id)
                            {
                                for mesh in &model.mesh_instances {
                                    cmd_buffer.cmd_bind_pipeline(wire_pso_depth_pipeline);

                                    let mesh = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                                    render_aabb_for_mesh(
                                        wireframe_cube,
                                        mesh,
                                        transform,
                                        cmd_buffer,
                                    );

                                    cmd_buffer.cmd_bind_pipeline(solid_pso_depth_pipeline);

                                    render_mesh(
                                        mesh,
                                        transform,
                                        Color::new(0, 127, 127, 127),
                                        1.0,
                                        cmd_buffer,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_debug_display(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        debug_display: &DebugDisplay,
        mesh_manager: &MeshManager,
        wire_pso_depth_handle: PipelineHandle,
    ) {
        cmd_buffer.with_label("Debug_Display", |cmd_buffer| {
            if let Some(pipeline) = render_context
                .pipeline_manager
                .get_pipeline(wire_pso_depth_handle)
            {
                cmd_buffer.cmd_bind_pipeline(pipeline);

                render_context.bind_default_descriptor_sets(cmd_buffer);

                debug_display.render_primitives(|primitive| {
                    match primitive.primitive_type {
                        DebugPrimitiveType::DefaultMesh { default_mesh_type } => {
                            render_mesh(
                                mesh_manager.get_default_mesh(default_mesh_type),
                                &primitive.transform,
                                primitive.color,
                                1.0,
                                cmd_buffer,
                            );
                        }
                    };
                });
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_manipulators(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        manipulator_meshes: &[(&GlobalTransform, &ManipulatorComponent)],
        mesh_manager: &MeshManager,
        render_viewport: &RenderViewport,
        render_camera: &RenderCamera,
        solid_pso_no_depth_handle: PipelineHandle,
    ) {
        for (transform, manipulator) in manipulator_meshes.iter() {
            if manipulator.active {
                cmd_buffer.with_label("Manipulator", |cmd_buffer| {
                    let view_transform = render_camera.view_transform();
                    let projection = render_camera.build_projection(
                        render_viewport.extents().width as f32,
                        render_viewport.extents().height as f32,
                    );
                    let scaled_xform = ManipulatorManager::scale_manipulator_for_viewport(
                        transform,
                        &manipulator.local_transform,
                        projection,
                        &view_transform,
                    );

                    let mut color = if manipulator.selected {
                        Color::YELLOW
                    } else {
                        manipulator.color
                    };
                    color.a = if manipulator.transparent { 225 } else { 255 };

                    if let Some(pipeline) = render_context
                        .pipeline_manager
                        .get_pipeline(solid_pso_no_depth_handle)
                    {
                        cmd_buffer.cmd_bind_pipeline(pipeline);

                        render_context.bind_default_descriptor_sets(cmd_buffer);

                        render_mesh(
                            mesh_manager.get_default_mesh(manipulator.default_mesh_type),
                            &scaled_xform,
                            color,
                            1.0,
                            cmd_buffer,
                        );
                    }
                });
            }
        }
    }
}

fn render_aabb_for_mesh(
    wire_frame_cube: &MeshMetaData,
    mesh: &MeshMetaData,
    transform: &GlobalTransform,
    cmd_buffer: &mut CommandBuffer,
) {
    cmd_buffer.with_label("AABB", |cmd_buffer| {
        let mut min_bound = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_bound = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for position in &mesh.positions {
            let world_pos = transform.mul_vec3(*position);

            min_bound = min_bound.min(world_pos);
            max_bound = max_bound.max(world_pos);
        }

        let delta = max_bound - min_bound;
        let mid_point = min_bound + delta * 0.5;

        let aabb_transform = GlobalTransform::identity()
            .with_translation(mid_point)
            .with_scale(delta);

        render_mesh(
            wire_frame_cube,
            &aabb_transform,
            Color::WHITE,
            1.0,
            cmd_buffer,
        );
    });
}

fn render_mesh(
    mesh_meta_data: &MeshMetaData,
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
    push_constant_data.set_mesh_description_offset(mesh_meta_data.mesh_description_offset.into());

    cmd_buffer.cmd_push_constant_typed(&push_constant_data);

    cmd_buffer.cmd_draw_indexed(
        mesh_meta_data.index_count.get(),
        mesh_meta_data.index_offset,
        0,
    );
}
