use std::num::NonZeroU32;

use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DescriptorDef, DescriptorRef,
    DescriptorSetLayoutDef, Format, GraphicsPipelineDef, LoadOp, Pipeline, PipelineType,
    PrimitiveTopology, PushConstantDef, RasterizerState, ResourceState, ResourceUsage,
    RootSignature, RootSignatureDef, SampleCount, ShaderPackage, ShaderStageDef, ShaderStageFlags,
    StencilOp, StoreOp, VertexLayout, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use lgn_math::{Mat4, Vec3};
use lgn_pso_compiler::{CompileParams, EntryPoint, ShaderSource};
use lgn_transform::prelude::Transform;

use crate::{
    components::{PickedComponent, RenderSurface, StaticMesh},
    hl_gfx_api::HLCommandBuffer,
    RenderContext, Renderer,
};

pub struct TmpRenderPass {
    root_signature: RootSignature,
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        //
        // Shaders
        //

        let shader_compiler = renderer.shader_compiler();

        let shader_build_result = shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Path(
                    "crate://renderer/shaders/shader.hlsl".to_owned(),
                ),
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
                rasterizer_state: &RasterizerState::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        Self {
            root_signature,
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

    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, Option<&PickedComponent>)],
        camera_transform: &Transform,
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
        let descriptor_set_layout = &self
            .pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0];

        let fov_y_radians: f32 = 45.0;
        let width = render_surface.extents().width() as f32;
        let height = render_surface.extents().height() as f32;
        let aspect_ratio: f32 = width / height;
        let z_near: f32 = 0.01;
        let z_far: f32 = 100.0;
        let projection_matrix = Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

        let view_matrix = Mat4::look_at_lh(
            camera_transform.translation,
            camera_transform.translation + camera_transform.forward(),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let transient_allocator = render_context.transient_buffer_allocator();

        for (_index, (static_mesh_component, picked_component)) in static_meshes.iter().enumerate()
        {
            let color: (f32, f32, f32, f32) = (
                f32::from(static_mesh_component.color.r) / 255.0f32,
                f32::from(static_mesh_component.color.g) / 255.0f32,
                f32::from(static_mesh_component.color.b) / 255.0f32,
                f32::from(static_mesh_component.color.a) / 255.0f32,
            );

            let mut constant_data: [f32; 36] = [0.0; 36];
            view_matrix.write_cols_to_slice(&mut constant_data[0..]);
            projection_matrix.write_cols_to_slice(&mut constant_data[16..]);
            constant_data[32] = color.0;
            constant_data[33] = color.1;
            constant_data[34] = color.2;
            constant_data[35] = 1.0;

            let sub_allocation =
                transient_allocator.copy_data(&constant_data, ResourceUsage::AS_CONST_BUFFER);

            let const_buffer_view = sub_allocation.const_buffer_view();

            /* WIP
            {
                let bump_allocator = render_context.bump_allocator();

                let descriptor_heap_partition = render_context
                    .descriptor_pool()
                    .descriptor_heap_partition_mut();

              let mut ds_data = FakeDescriptorSetData::new(
                    render_context.cgen_runtime(),
                    bump_allocator.bumpalo(),
                    descriptor_heap_partition,
                );


                ds_data.set_constant_buffer(FakeDescriptorID::A, &const_buffer_view);

                let descriptor_handle = ds_data.build(render_context.renderer().device_context());
            }
            */

            let mut descriptor_set_writer =
                render_context.alloc_descriptor_set(descriptor_set_layout);

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

            let mut push_constant_data: [u32; 3] = [0; 3];
            push_constant_data[0] = static_mesh_component.vertex_offset;
            push_constant_data[1] = static_mesh_component.world_offset;
            push_constant_data[2] = if picked_component.is_some() { 1 } else { 0 };

            cmd_buffer.push_constants(&self.root_signature, &push_constant_data);

            cmd_buffer.draw(static_mesh_component.num_verticies, 0);
        }

        cmd_buffer.end_render_pass();
    }
}
