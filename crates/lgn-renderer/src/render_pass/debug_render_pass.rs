use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, FillMode, Format, GraphicsPipelineDef,
    LoadOp, PrimitiveTopology, RasterizerState, SampleCount, StencilOp, StoreOp, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::{Mat4, Vec3, Vec4, Vec4Swizzles};

use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen,
    components::{CameraComponent, ManipulatorComponent, RenderSurface, VisualComponent},
    debug_display::{DebugDisplay, DebugPrimitiveType},
    hl_gfx_api::HLCommandBuffer,
    picking::ManipulatorManager,
    resources::{DefaultMeshType, MeshManager, PipelineHandle, PipelineManager},
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
        cmd_buffer: &mut HLCommandBuffer<'_>,
        mesh_manager: &MeshManager,
    ) {
        let wire_pso_depth_pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.wire_pso_depth_handle)
            .unwrap();
        cmd_buffer.bind_pipeline(wire_pso_depth_pipeline);
        cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

        render_mesh(
            DefaultMeshType::GroundPlane as u32,
            &Mat4::IDENTITY,
            Vec4::ZERO,
            cmd_buffer,
            mesh_manager,
        );
    }

    pub fn render_picked(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        picked_meshes: &[(&VisualComponent, &GlobalTransform)],
        mesh_manager: &MeshManager,
    ) {
        cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

        let wire_pso_depth_pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.wire_pso_depth_handle)
            .unwrap();
        let solid_pso_depth_pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.solid_pso_depth_handle)
            .unwrap();
        for (_index, (static_mesh_component, transform)) in picked_meshes.iter().enumerate() {
            cmd_buffer.bind_pipeline(wire_pso_depth_pipeline);
            render_aabb_for_mesh(
                static_mesh_component.mesh_id as u32,
                transform,
                cmd_buffer,
                mesh_manager,
            );
            cmd_buffer.bind_pipeline(solid_pso_depth_pipeline);
            render_mesh(
                static_mesh_component.mesh_id as u32,
                &transform.compute_matrix(),
                Vec4::new(0.0, 0.5, 0.5, 0.75),
                cmd_buffer,
                mesh_manager,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_debug_display(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        debug_display: &mut DebugDisplay,
        mesh_manager: &MeshManager,
    ) {
        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.wire_pso_depth_handle)
            .unwrap();
        cmd_buffer.bind_pipeline(pipeline);
        cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

        debug_display.render_primitives(|primitive| {
            let mesh_id = match primitive.primitive_type {
                DebugPrimitiveType::Mesh { mesh_id } => mesh_id,
            };

            render_mesh(
                mesh_id as u32,
                &primitive.transform,
                primitive.color.extend(1.0),
                cmd_buffer,
                mesh_manager,
            );
        });
    }

    fn render_manipulators(
        &self,
        render_context: &RenderContext<'_>,

        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        manipulator_meshes: &[(&VisualComponent, &GlobalTransform, &ManipulatorComponent)],
        mesh_manager: &MeshManager,
        camera: &CameraComponent,
    ) {
        let (view_matrix, projection_matrix) = camera.build_view_projection(
            render_surface.extents().width() as f32,
            render_surface.extents().height() as f32,
        );

        for (_index, (static_mesh, transform, manipulator)) in manipulator_meshes.iter().enumerate()
        {
            if manipulator.active {
                let scaled_world_matrix = ManipulatorManager::scale_manipulator_for_viewport(
                    transform,
                    &manipulator.local_transform,
                    &view_matrix,
                    &projection_matrix,
                );

                let mut color = if manipulator.selected {
                    Vec4::new(1.0, 1.0, 0.0, 1.0)
                } else {
                    Vec4::new(
                        f32::from(static_mesh.color.r) / 255.0f32,
                        f32::from(static_mesh.color.g) / 255.0f32,
                        f32::from(static_mesh.color.b) / 255.0f32,
                        f32::from(static_mesh.color.a) / 255.0f32,
                    )
                };

                color.w = if manipulator.transparent { 0.9 } else { 1.0 };

                let pipeline = render_context
                    .pipeline_manager()
                    .get_pipeline(self.solid_pso_nodepth_handle)
                    .unwrap();
                cmd_buffer.bind_pipeline(pipeline);
                cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
                cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

                render_mesh(
                    static_mesh.mesh_id as u32,
                    &scaled_world_matrix,
                    color,
                    cmd_buffer,
                    mesh_manager,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        picked_meshes: &[(&VisualComponent, &GlobalTransform)],
        manipulator_meshes: &[(&VisualComponent, &GlobalTransform, &ManipulatorComponent)],
        camera: &CameraComponent,
        mesh_manager: &MeshManager,
        debug_display: &mut DebugDisplay,
    ) {
        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_surface.render_target_view(),
                load_op: LoadOp::Load,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue::default(),
            }],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_stencil_texture_view(),
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

        self.render_picked(render_context, cmd_buffer, picked_meshes, mesh_manager);

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
    }
}

#[allow(clippy::too_many_arguments)]
fn render_aabb_for_mesh(
    mesh_id: u32,
    transform: &GlobalTransform,
    cmd_buffer: &mut HLCommandBuffer<'_>,
    mesh_manager: &MeshManager,
) {
    let mesh = mesh_manager.mesh_from_id(mesh_id);

    let mut min_bound = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max_bound = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for position in mesh.positions.as_ref().unwrap() {
        let world_pos = transform.compute_matrix().mul_vec4(*position).xyz();

        min_bound = min_bound.min(world_pos);
        max_bound = max_bound.max(world_pos);
    }

    let delta = max_bound - min_bound;
    let mid_point = min_bound + delta * 0.5;

    let aabb_transform = GlobalTransform::identity()
        .with_translation(mid_point)
        .with_scale(delta);

    render_mesh(
        DefaultMeshType::WireframeCube as u32,
        &aabb_transform.compute_matrix(),
        Vec4::new(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        cmd_buffer,
        mesh_manager,
    );
}

fn render_mesh(
    mesh_id: u32,
    world_xform: &Mat4,
    color: Vec4,
    cmd_buffer: &HLCommandBuffer<'_>,
    mesh_manager: &MeshManager,
) {
    let mut push_constant_data = cgen::cgen_type::ConstColorPushConstantData::default();

    push_constant_data.set_world((*world_xform).into());
    push_constant_data.set_color(color.into());
    push_constant_data
        .set_mesh_description_offset(mesh_manager.mesh_description_offset_from_id(mesh_id).into());

    cmd_buffer.push_constant(&push_constant_data);

    cmd_buffer.draw(mesh_manager.mesh_from_id(mesh_id).num_vertices() as u32, 0);
}
