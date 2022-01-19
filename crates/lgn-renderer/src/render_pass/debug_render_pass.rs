use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, FillMode, Format, GraphicsPipelineDef,
    LoadOp, Pipeline, PrimitiveTopology, RasterizerState, SampleCount, StencilOp, StoreOp,
    VertexLayout,
};
use lgn_math::{Mat4, Vec3, Vec4, Vec4Swizzles};

use lgn_transform::prelude::Transform;

use crate::{
    cgen,
    components::{
        CameraComponent, ManipulatorComponent, PickedComponent, RenderSurface, StaticMesh,
    },
    debug_display::{DebugDisplay, DebugPrimitiveType},
    hl_gfx_api::HLCommandBuffer,
    picking::ManipulatorManager,
    resources::{DefaultMeshId, DefaultMeshes},
    RenderContext, Renderer,
};

pub struct DebugRenderPass {
    _solid_pso_depth: Pipeline,
    wire_pso_depth: Pipeline,
    solid_pso_nodepth: Pipeline,
    _wire_pso_nodepth: Pipeline,
}

impl DebugRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        let shader =
            renderer.prepare_vs_ps_no_rs(String::from("crate://renderer/shaders/const_color.hlsl"));

        //
        // Pipeline state
        //
        let vertex_layout = VertexLayout {
            attributes: vec![],
            buffers: vec![],
        };

        let depth_state_enabled = DepthState {
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

        let wire_frame_state = RasterizerState {
            fill_mode: FillMode::Wireframe,
            ..RasterizerState::default()
        };

        let root_signature = cgen::pipeline_layout::ConstColorPipelineLayout::root_signature();

        let solid_pso_depth = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
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
            .unwrap();

        let wire_pso_depth = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
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
            .unwrap();

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

        let solid_pso_nodepth = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
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
            .unwrap();

        let wire_pso_nodepth = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
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
            .unwrap();

        Self {
            _solid_pso_depth: solid_pso_depth,
            wire_pso_depth,
            solid_pso_nodepth,
            _wire_pso_nodepth: wire_pso_nodepth,
        }
    }
    /*
        pub fn bind_pipeline_and_desc_set(
            pipeline: &Pipeline,
            // frame_descriptor_set_handle: DescriptorSetHandle,
            // view_descriptor_set_handle: DescriptorSetHandle,
            // view_data: &cgen::cgen_type::ViewData,
            // constant_data: &cgen::cgen_type::ConstData,
            cmd_buffer: &mut HLCommandBuffer<'_>,
            render_context: &RenderContext<'_>,
        ) {
            cmd_buffer.bind_pipeline(pipeline);
            cmd_buffer.bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
            cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

            // let descriptor_set_layout = &pipeline
            //     .root_signature()
            //     .definition()
            //     .descriptor_set_layouts[0];

            // let mut descriptor_set_writer = render_context.alloc_descriptor_set(descriptor_set_layout);

            // {
            //     let sub_allocation =
            //         transient_allocator.copy_data(view_data, ResourceUsage::AS_CONST_BUFFER);

            //     let const_buffer_view = sub_allocation.const_buffer_view();

            //     descriptor_set_writer
            //         .set_descriptors_by_name(
            //             "view_data",
            //             &[DescriptorRef::BufferView(&const_buffer_view)],
            //         )
            //         .unwrap();
            // }
            // {
            //     let sub_allocation =
            //         transient_allocator.copy_data(constant_data, ResourceUsage::AS_CONST_BUFFER);

            //     let const_buffer_view = sub_allocation.const_buffer_view();

            //     descriptor_set_writer
            //         .set_descriptors_by_name(
            //             "const_data",
            //             &[DescriptorRef::BufferView(&const_buffer_view)],
            //         )
            //         .unwrap();
            // }

            // let static_buffer_ro_view = render_context.renderer().static_buffer_ro_view();
            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "static_buffer",
            //         &[DescriptorRef::BufferView(&static_buffer_ro_view)],
            //     )
            //     .unwrap();

            // let descriptor_set_handle =
            //     descriptor_set_writer.flush(render_context.renderer().device_context());

            // cmd_buffer.bind_descriptor_set_handle(
            //     PipelineType::Graphics,
            //     pipeline.root_signature(),
            //     descriptor_set_layout.definition().frequency,
            //     descriptor_set_handle,
            // );
        }
    */

    pub fn render_ground_plane(
        &self,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_context: &RenderContext<'_>,
        default_meshes: &DefaultMeshes,
    ) {
        // let mut constant_data = cgen::cgen_type::ConstData::default();
        // constant_data.set_world(Mat4::IDENTITY.into());
        // constant_data.set_color(Vec4::ZERO.into());

        cmd_buffer.bind_pipeline(&self.wire_pso_depth);
        cmd_buffer.bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

        // self.bind_pipeline_and_desc_set(
        //     &self.wire_pso_depth,
        //     // view_data,
        //     // &constant_data,
        //     cmd_buffer,
        //     render_context,
        // );

        render_mesh(
            DefaultMeshId::GroundPlane as u32,
            &Mat4::IDENTITY,
            Vec4::ZERO,
            cmd_buffer,
            default_meshes,
        );
    }

    pub fn render_aabbs(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        static_meshes: &[(&StaticMesh, &Transform, Option<&PickedComponent>)],
        default_meshes: &DefaultMeshes,
    ) {
        cmd_buffer.bind_pipeline(&self.wire_pso_depth);
        cmd_buffer.bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

        for (_index, (static_mesh_component, transform, picked)) in static_meshes.iter().enumerate()
        {
            if picked.is_some() {
                render_aabb_for_mesh(
                    static_mesh_component.mesh_id as u32,
                    transform,
                    cmd_buffer,
                    default_meshes,
                );
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_debug_display(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        debug_display: &mut DebugDisplay,
        default_meshes: &DefaultMeshes,
    ) {
        cmd_buffer.bind_pipeline(&self.wire_pso_depth);
        cmd_buffer.bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

        debug_display.render_primitives(|primitive| {
            let mesh_id = match primitive.primitive_type {
                DebugPrimitiveType::Mesh { mesh_id } => mesh_id,
            };

            // let mut constant_data = cgen::cgen_type::ConstData::default();
            // constant_data.set_world(primitive.transform.into());
            // constant_data.set_color(
            //     Vec4::new(primitive.color.0, primitive.color.1, primitive.color.2, 1.0).into(),
            // );

            // self.bind_pipeline_and_desc_set(
            //     &self.wire_pso_depth,
            //     // view_data,
            //     // &constant_data,
            //     cmd_buffer,
            //     render_context,
            // );

            render_mesh(
                mesh_id as u32,
                &primitive.transform,
                primitive.color.extend(1.0),
                cmd_buffer,
                default_meshes,
            );
        });

        debug_display.clear_display_lists();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, &Transform, Option<&PickedComponent>)],
        manipulator_meshes: &[(&StaticMesh, &Transform, &ManipulatorComponent)],
        camera: &CameraComponent,
        default_meshes: &DefaultMeshes,
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

        let (view_matrix, projection_matrix) = camera.build_view_projection(
            render_surface.extents().width() as f32,
            render_surface.extents().height() as f32,
        );

        self.render_ground_plane(cmd_buffer, render_context, default_meshes);

        for (_index, (static_mesh, transform, manipulator)) in manipulator_meshes.iter().enumerate()
        {
            if manipulator.active {
                let scaled_world_matrix = ManipulatorManager::scale_manipulator_for_viewport(
                    transform,
                    &manipulator.local_transform,
                    &view_matrix,
                    &projection_matrix,
                );

                let mut color = Vec4::new(
                    f32::from(static_mesh.color.r) / 255.0f32,
                    f32::from(static_mesh.color.g) / 255.0f32,
                    f32::from(static_mesh.color.b) / 255.0f32,
                    f32::from(static_mesh.color.a) / 255.0f32,
                );

                if manipulator.selected {
                    color = Vec4::new(1.0, 1.0, 0.0, 1.0);
                }

                color.w = if manipulator.transparent { 0.9 } else { 1.0 };

                // let mut constant_data = cgen::cgen_type::ConstData::default();
                // constant_data.set_world(scaled_world_matrix.into());
                // constant_data.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());

                cmd_buffer.bind_pipeline(&self.solid_pso_nodepth);
                cmd_buffer
                    .bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
                cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

                // self.bind_pipeline_and_desc_set(
                //     &self.solid_pso_nodepth,
                //     // &view_data,
                //     // &constant_data,
                //     cmd_buffer,
                //     render_context,
                // );

                render_mesh(
                    static_mesh.mesh_id as u32,
                    &scaled_world_matrix,
                    color,
                    cmd_buffer,
                    default_meshes,
                );
            }
        }

        self.render_aabbs(render_context, cmd_buffer, static_meshes, default_meshes);

        self.render_debug_display(render_context, cmd_buffer, debug_display, default_meshes);

        cmd_buffer.end_render_pass();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_aabb_for_mesh(
    mesh_id: u32,
    transform: &Transform,
    cmd_buffer: &mut HLCommandBuffer<'_>,
    default_meshes: &DefaultMeshes,
) {
    let mesh = default_meshes.mesh_from_id(mesh_id);

    let mut min_bound = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max_bound = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for i in 0..mesh.num_vertices() {
        let position = Vec4::new(
            mesh.vertices[i * 14],
            mesh.vertices[i * 14 + 1],
            mesh.vertices[i * 14 + 2],
            1.0,
        );

        let world_pos = transform.compute_matrix().mul_vec4(position).xyz();

        min_bound = min_bound.min(world_pos);
        max_bound = max_bound.max(world_pos);
    }

    let delta = max_bound - min_bound;
    let mid_point = min_bound + delta * 0.5;

    let aabb_transform = Transform::identity()
        .with_translation(mid_point)
        .with_scale(delta);

    // let mut constant_data = cgen::cgen_type::ConstData::default();
    // constant_data.set_world(aabb_transform.compute_matrix().into());
    // constant_data.set_color(Vec4::new(1.0f32, 1.0f32, 0.0f32, 1.0f32).into());

    // self.bind_pipeline_and_desc_set(
    //     &self.wire_pso_depth,
    //     // view_data,
    //     // &constant_data,
    //     cmd_buffer,
    //     render_context,
    // );

    render_mesh(
        DefaultMeshId::WireframeCube as u32,
        &aabb_transform.compute_matrix(),
        Vec4::new(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        cmd_buffer,
        default_meshes,
    );
}

fn render_mesh(
    mesh_id: u32,
    world_xform: &Mat4,
    color: Vec4,
    cmd_buffer: &HLCommandBuffer<'_>,
    default_meshes: &DefaultMeshes,
) {
    let mut push_constant_data = cgen::cgen_type::ConstColorPushConstantData::default();

    push_constant_data.set_world((*world_xform).into());
    push_constant_data.set_color(color.into());
    push_constant_data.set_vertex_offset(default_meshes.mesh_offset_from_id(mesh_id).into());

    cmd_buffer.push_constant2(&push_constant_data);

    cmd_buffer.draw(
        default_meshes.mesh_from_id(mesh_id).num_vertices() as u32,
        0,
    );
}
