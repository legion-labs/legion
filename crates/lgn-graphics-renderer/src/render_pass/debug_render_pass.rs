use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, FillMode, Format, GraphicsPipelineDef,
    LoadOp, PrimitiveTopology, RasterizerState, SampleCount, StencilOp, StoreOp, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_graphics_data::Color;
use lgn_math::{Vec3, Vec4};

use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen::{self, cgen_type::TransformData},
    components::{CameraComponent, ManipulatorComponent, RenderSurface, VisualComponent},
    debug_display::{DebugDisplay, DebugPrimitiveType},
    hl_gfx_api::HLCommandBuffer,
    picking::ManipulatorManager,
    resources::{
        DefaultMeshType, MeshManager, MeshMetaData, ModelManager, PipelineHandle, PipelineManager,
    },
    RenderContext,
};

pub struct DebugRenderPass {
    solid_pso_depth_handle: PipelineHandle,
    wire_pso_depth_handle: PipelineHandle,
    solid_pso_nodepth_handle: PipelineHandle,
    _wire_pso_nodepth_handle: PipelineHandle,
}

impl DebugRenderPass {
    pub fn new(pipeline_manager: &PipelineManager) -> Self {
        let root_signature = cgen::pipeline_layout::ConstColorPipelineLayout::root_signature();

        let vertex_layout = VertexLayout::default();

        let depth_state_enabled = DepthState {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_op: CompareOp::LessOrEqual,
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

        let wire_frame_state = RasterizerState {
            fill_mode: FillMode::Wireframe,
            ..RasterizerState::default()
        };

        let solid_pso_depth_handle = pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::const_color_shader::ID,
                cgen::shader::const_color_shader::NONE,
            ),
            move |device_context, shader| {
                device_context
                    .create_graphics_pipeline(&GraphicsPipelineDef {
                        shader,
                        root_signature,
                        vertex_layout: &vertex_layout,
                        blend_state: &BlendState::default_alpha_enabled(),
                        depth_state: &depth_state_enabled,
                        rasterizer_state: &RasterizerState::default(),
                        color_formats: &[Format::R16G16B16A16_SFLOAT],
                        sample_count: SampleCount::SampleCount1,
                        depth_stencil_format: Some(Format::D32_SFLOAT),
                        primitive_topology: PrimitiveTopology::TriangleList,
                    })
                    .unwrap()
            },
        );

        let wire_pso_depth_handle = pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::const_color_shader::ID,
                cgen::shader::const_color_shader::NONE,
            ),
            move |device_context, shader| {
                device_context
                    .create_graphics_pipeline(&GraphicsPipelineDef {
                        shader,
                        root_signature,
                        vertex_layout: &vertex_layout,
                        blend_state: &BlendState::default_alpha_enabled(),
                        depth_state: &depth_state_enabled,
                        rasterizer_state: &wire_frame_state,
                        color_formats: &[Format::R16G16B16A16_SFLOAT],
                        sample_count: SampleCount::SampleCount1,
                        depth_stencil_format: Some(Format::D32_SFLOAT),
                        primitive_topology: PrimitiveTopology::LineList,
                    })
                    .unwrap()
            },
        );

        let solid_pso_nodepth_handle = pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::const_color_shader::ID,
                cgen::shader::const_color_shader::NONE,
            ),
            move |device_context, shader| {
                device_context
                    .create_graphics_pipeline(&GraphicsPipelineDef {
                        shader,
                        root_signature,
                        vertex_layout: &vertex_layout,
                        blend_state: &BlendState::default_alpha_enabled(),
                        depth_state: &depth_state_disabled,
                        rasterizer_state: &RasterizerState::default(),
                        color_formats: &[Format::R16G16B16A16_SFLOAT],
                        sample_count: SampleCount::SampleCount1,
                        depth_stencil_format: Some(Format::D32_SFLOAT),
                        primitive_topology: PrimitiveTopology::TriangleList,
                    })
                    .unwrap()
            },
        );

        let wire_pso_nodepth_handle = pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::const_color_shader::ID,
                cgen::shader::const_color_shader::NONE,
            ),
            move |device_context, shader| {
                device_context
                    .create_graphics_pipeline(&GraphicsPipelineDef {
                        shader,
                        root_signature,
                        vertex_layout: &vertex_layout,
                        blend_state: &BlendState::default_alpha_enabled(),
                        depth_state: &depth_state_disabled,
                        rasterizer_state: &wire_frame_state,
                        color_formats: &[Format::R16G16B16A16_SFLOAT],
                        sample_count: SampleCount::SampleCount1,
                        depth_stencil_format: Some(Format::D32_SFLOAT),
                        primitive_topology: PrimitiveTopology::LineList,
                    })
                    .unwrap()
            },
        );

        Self {
            solid_pso_depth_handle,
            wire_pso_depth_handle,
            solid_pso_nodepth_handle,
            _wire_pso_nodepth_handle: wire_pso_nodepth_handle,
        }
    }

    pub fn render_ground_plane(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer,
        mesh_manager: &MeshManager,
    ) {
        cmd_buffer.with_label("Ground Plane", |cmd_buffer| {
            let wire_pso_depth_pipeline = render_context
                .pipeline_manager()
                .get_pipeline(self.wire_pso_depth_handle)
                .unwrap();
            cmd_buffer.bind_pipeline(wire_pso_depth_pipeline);

            render_context.bind_default_descriptor_sets(cmd_buffer);

            render_mesh(
                mesh_manager.get_default_mesh(DefaultMeshType::GroundPlane),
                &GlobalTransform::identity(),
                Color::BLACK,
                0.0,
                cmd_buffer,
            );
        });
    }

    pub fn render_picked(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer,
        picked_meshes: &[(&VisualComponent, &GlobalTransform)],
        mesh_manager: &MeshManager,
        model_manager: &ModelManager,
    ) {
        cmd_buffer.with_label("Picked", |cmd_buffer| {
            render_context.bind_default_descriptor_sets(cmd_buffer);

            let wire_pso_depth_pipeline = render_context
                .pipeline_manager()
                .get_pipeline(self.wire_pso_depth_handle)
                .unwrap();
            let solid_pso_depth_pipeline = render_context
                .pipeline_manager()
                .get_pipeline(self.solid_pso_depth_handle)
                .unwrap();

            let wireframe_cube = mesh_manager.get_default_mesh(DefaultMeshType::WireframeCube);
            for (visual_component, transform) in picked_meshes.iter() {
                if let Some(model_resource_id) = visual_component.model_resource_id() {
                    if let Some(model) = model_manager.get_model_meta_data(model_resource_id) {
                        for mesh in &model.mesh_instances {
                            cmd_buffer.bind_pipeline(wire_pso_depth_pipeline);

                            let mesh = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                            render_aabb_for_mesh(wireframe_cube, mesh, transform, cmd_buffer);

                            cmd_buffer.bind_pipeline(solid_pso_depth_pipeline);

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
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_debug_display(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer,
        debug_display: &DebugDisplay,
        mesh_manager: &MeshManager,
    ) {
        cmd_buffer.with_label("Debug_Display", |cmd_buffer| {
            let pipeline = render_context
                .pipeline_manager()
                .get_pipeline(self.wire_pso_depth_handle)
                .unwrap();
            cmd_buffer.bind_pipeline(pipeline);

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
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_manipulators(
        &self,
        render_context: &RenderContext<'_>,

        cmd_buffer: &mut HLCommandBuffer,
        render_surface: &mut RenderSurface,
        manipulator_meshes: &[(&GlobalTransform, &ManipulatorComponent)],
        mesh_manager: &MeshManager,
        camera: &CameraComponent,
    ) {
        for (transform, manipulator) in manipulator_meshes.iter() {
            if manipulator.active {
                cmd_buffer.with_label("Manipulator", |cmd_buffer| {
                    let scaled_xform = ManipulatorManager::scale_manipulator_for_viewport(
                        transform,
                        &manipulator.local_transform,
                        render_surface,
                        camera,
                    );

                    let mut color = if manipulator.selected {
                        Color::YELLOW
                    } else {
                        manipulator.color
                    };
                    color.a = if manipulator.transparent { 225 } else { 255 };

                    let pipeline = render_context
                        .pipeline_manager()
                        .get_pipeline(self.solid_pso_nodepth_handle)
                        .unwrap();
                    cmd_buffer.bind_pipeline(pipeline);

                    render_context.bind_default_descriptor_sets(cmd_buffer);

                    render_mesh(
                        mesh_manager.get_default_mesh(manipulator.default_mesh_type),
                        &scaled_xform,
                        color,
                        1.0,
                        cmd_buffer,
                    );
                });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer,
        render_surface: &mut RenderSurface,
        picked_meshes: &[(&VisualComponent, &GlobalTransform)],
        manipulator_meshes: &[(&GlobalTransform, &ManipulatorComponent)],
        camera: &CameraComponent,
        mesh_manager: &MeshManager,
        model_manager: &ModelManager,
        debug_display: &DebugDisplay,
    ) {
        cmd_buffer.with_label("Debug", |cmd_buffer| {
            cmd_buffer.bind_index_buffer(render_context.static_buffer().index_buffer_binding());

            cmd_buffer.begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.hdr_rt().rtv(),
                    load_op: LoadOp::Load,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue::default(),
                }],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: render_surface.depth_rt().rtv(),
                    depth_load_op: LoadOp::Load,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::Store,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                }),
            );

            self.render_ground_plane(render_context, cmd_buffer, mesh_manager);

            self.render_picked(
                render_context,
                cmd_buffer,
                picked_meshes,
                mesh_manager,
                model_manager,
            );

            self.render_debug_display(render_context, cmd_buffer, debug_display, mesh_manager);

            self.render_manipulators(
                render_context,
                cmd_buffer,
                render_surface,
                manipulator_meshes,
                mesh_manager,
                camera,
            );

            cmd_buffer.end_render_pass();
        });
    }
}

fn render_aabb_for_mesh(
    wire_frame_cube: &MeshMetaData,
    mesh: &MeshMetaData,
    transform: &GlobalTransform,
    cmd_buffer: &mut HLCommandBuffer,
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
    cmd_buffer: &mut HLCommandBuffer,
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

    cmd_buffer.push_constant(&push_constant_data);

    if mesh_meta_data.index_count != 0 {
        cmd_buffer.draw_indexed(mesh_meta_data.index_count, mesh_meta_data.index_offset, 0);
    } else {
        cmd_buffer.draw(mesh_meta_data.vertex_count, 0);
    }
}
