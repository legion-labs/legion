use std::num::NonZeroU32;

use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DescriptorDef, DescriptorRef,
    DescriptorSetLayoutDef, FillMode, Format, GraphicsPipelineDef, LoadOp, Pipeline, PipelineType,
    PrimitiveTopology, PushConstantDef, RasterizerState, ResourceUsage, RootSignature,
    RootSignatureDef, SampleCount, ShaderPackage, ShaderStageDef, ShaderStageFlags, StencilOp,
    StoreOp, VertexLayout, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use lgn_math::{Mat4, Vec3};
use lgn_pso_compiler::{CompileParams, EntryPoint, ShaderSource};
use lgn_transform::prelude::Transform;

use crate::{
    components::{CameraComponent, PickedComponent, RenderSurface, StaticMesh},
    hl_gfx_api::HLCommandBuffer,
    resources::DefaultMeshes,
    RenderContext, Renderer,
};

pub struct DebugRenderPass {
    root_signature: RootSignature,
    _solid_pso_depth: Pipeline,
    wire_pso_depth: Pipeline,
    _solid_pso_nodepth: Pipeline,
    _wire_pso_nodepth: Pipeline,
}

impl DebugRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        //
        // Shaders
        //
        let shader_compiler = renderer.shader_compiler();

        let shader_source =
            String::from_utf8(include_bytes!("../../shaders/const_color.hlsl").to_vec()).unwrap();

        let shader_build_result = shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Code(shader_source),
                glob_defines: Vec::new(),
                entry_points: vec![
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_vs".to_owned(),
                        target_profile: "vs_6_0".to_owned(),
                    },
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_ps".to_owned(),
                        target_profile: "ps_6_0".to_owned(),
                    },
                ],
            })
            .unwrap();

        let vert_shader_module = device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let frag_shader_module = device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[1].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let shader = device_context
            .create_shader(
                vec![
                    ShaderStageDef {
                        entry_point: "main_vs".to_owned(),
                        shader_stage: ShaderStageFlags::VERTEX,
                        shader_module: vert_shader_module,
                    },
                    ShaderStageDef {
                        entry_point: "main_ps".to_owned(),
                        shader_stage: ShaderStageFlags::FRAGMENT,
                        shader_module: frag_shader_module,
                    },
                ],
                &shader_build_result.pipeline_reflection,
            )
            .unwrap();

        //
        // Root signature
        //
        let mut descriptor_set_layouts = Vec::new();
        for set_index in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
            let shader_resources: Vec<_> = shader_build_result
                .pipeline_reflection
                .shader_resources
                .iter()
                .filter(|x| x.set_index as usize == set_index)
                .collect();

            if !shader_resources.is_empty() {
                let descriptor_defs = shader_resources
                    .iter()
                    .map(|sr| DescriptorDef {
                        name: sr.name.clone(),
                        binding: sr.binding,
                        shader_resource_type: sr.shader_resource_type,
                        array_size: sr.element_count,
                    })
                    .collect();

                let def = DescriptorSetLayoutDef {
                    frequency: set_index as u32,
                    descriptor_defs,
                };
                let descriptor_set_layout =
                    device_context.create_descriptorset_layout(&def).unwrap();
                descriptor_set_layouts.push(descriptor_set_layout);
            }
        }

        let root_signature_def = RootSignatureDef {
            descriptor_set_layouts: descriptor_set_layouts.clone(),
            push_constant_def: shader_build_result
                .pipeline_reflection
                .push_constant
                .map(|x| PushConstantDef {
                    used_in_shader_stages: x.used_in_shader_stages,
                    size: NonZeroU32::new(x.size).unwrap(),
                }),
        };

        let root_signature = device_context
            .create_root_signature(&root_signature_def)
            .unwrap();

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

        let solid_pso_depth = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
                root_signature: &root_signature,
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
                root_signature: &root_signature,
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
                root_signature: &root_signature,
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
                root_signature: &root_signature,
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
            root_signature,
            _solid_pso_depth: solid_pso_depth,
            wire_pso_depth,
            _solid_pso_nodepth: solid_pso_nodepth,
            _wire_pso_nodepth: wire_pso_nodepth,
        }
    }

    pub fn bind_pipeline_and_desc_set(
        &self,
        pipeline: &Pipeline,
        constant_data: [f32; 53],
        cmd_buffer: &HLCommandBuffer<'_>,
        render_context: &RenderContext<'_>,
    ) {
        cmd_buffer.bind_pipeline(pipeline);
        let descriptor_set_layout = &pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0];

        let mut descriptor_set_writer = render_context.alloc_descriptor_set(descriptor_set_layout);

        let transient_allocator = render_context.transient_buffer_allocator();
        let sub_allocation =
            transient_allocator.copy_data(&constant_data, ResourceUsage::AS_CONST_BUFFER);

        let const_buffer_view = sub_allocation.const_buffer_view();

        descriptor_set_writer
            .set_descriptors_by_name(
                "const_data",
                &[DescriptorRef::BufferView(&const_buffer_view)],
            )
            .unwrap();

        let static_buffer_ro_view = render_context.renderer().static_buffer_ro_view();
        descriptor_set_writer
            .set_descriptors_by_name(
                "static_buffer",
                &[DescriptorRef::BufferView(&static_buffer_ro_view)],
            )
            .unwrap();

        let descriptor_set_handle =
            descriptor_set_writer.flush(render_context.renderer().device_context());

        cmd_buffer.bind_descriptor_set_handle(
            PipelineType::Graphics,
            &self.root_signature,
            descriptor_set_layout.definition().frequency,
            descriptor_set_handle,
        );
    }

    pub fn render_mesh(
        &self,
        mesh_id: u32,
        cmd_buffer: &HLCommandBuffer<'_>,
        default_meshes: &DefaultMeshes,
    ) {
        let mut push_constant_data: [u32; 1] = [0; 1];
        push_constant_data[0] = default_meshes.mesh_offset_from_id(mesh_id);

        cmd_buffer.push_constants(&self.root_signature, &push_constant_data);

        cmd_buffer.draw(
            default_meshes.mesh_from_id(mesh_id).num_vertices() as u32,
            0,
        );
    }

    pub fn render_ground_plane(
        &self,
        mut constant_data: [f32; 53],
        cmd_buffer: &HLCommandBuffer<'_>,
        render_context: &RenderContext<'_>,
        default_meshes: &DefaultMeshes,
    ) {
        Mat4::IDENTITY.write_cols_to_slice(&mut constant_data[0..]);

        constant_data[48] = 0.0;
        constant_data[49] = 0.0;
        constant_data[50] = 0.0;
        constant_data[51] = 0.0;
        constant_data[52] = 0.0;

        self.bind_pipeline_and_desc_set(
            &self.wire_pso_depth,
            constant_data,
            cmd_buffer,
            render_context,
        );

        self.render_mesh(4, cmd_buffer, default_meshes);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_aabb_for_mesh(
        &self,
        mesh_id: u32,
        transform: &Transform,
        mut constant_data: [f32; 53],
        cmd_buffer: &HLCommandBuffer<'_>,
        render_context: &RenderContext<'_>,
        default_meshes: &DefaultMeshes,
    ) {
        let mesh = default_meshes.mesh_from_id(mesh_id);

        let mut min_bound = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_bound = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for i in 0..mesh.num_vertices() {
            let position = Vec3::new(
                mesh.vertices[i * 14],
                mesh.vertices[i * 14 + 1],
                mesh.vertices[i * 14 + 2],
            );

            let world_pos = transform.mul_vec3(position);

            min_bound = min_bound.min(world_pos);
            max_bound = max_bound.max(world_pos);
        }

        let delta = max_bound - min_bound;
        let mid_point = min_bound + delta * 0.5;

        let aabb_transform = Transform::identity()
            .with_translation(mid_point)
            .with_scale(delta);

        aabb_transform
            .compute_matrix()
            .write_cols_to_slice(&mut constant_data[0..]);

        constant_data[48] = 1.0;
        constant_data[49] = 1.0;
        constant_data[50] = 0.0;
        constant_data[51] = 1.0;
        constant_data[52] = 0.0;

        self.bind_pipeline_and_desc_set(
            &self.wire_pso_depth,
            constant_data,
            cmd_buffer,
            render_context,
        );

        self.render_mesh(3, cmd_buffer, default_meshes);
    }

    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, &Transform, &PickedComponent)],
        camera: &CameraComponent,
        default_meshes: &DefaultMeshes,
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

        let mut constant_data: [f32; 53] = [0.0; 53];
        view_matrix.write_cols_to_slice(&mut constant_data[16..]);
        projection_matrix.write_cols_to_slice(&mut constant_data[32..]);

        self.render_ground_plane(constant_data, cmd_buffer, render_context, default_meshes);

        for (_index, (static_mesh_component, transform, _picked_component)) in
            static_meshes.iter().enumerate()
        {
            self.render_aabb_for_mesh(
                static_mesh_component.mesh_id as u32,
                transform,
                constant_data,
                cmd_buffer,
                render_context,
                default_meshes,
            );
        }

        cmd_buffer.end_render_pass();
    }
}
