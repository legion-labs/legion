#![allow(unsafe_code)]

use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DescriptorRef, Format,
    GraphicsPipelineDef, LoadOp, Pipeline, PipelineType, PrimitiveTopology, RasterizerState,
    ResourceState, ResourceUsage, RootSignature, SampleCount, StencilOp, StoreOp, VertexLayout,
};
use lgn_transform::prelude::Transform;

use crate::{
    components::{
        CameraComponent, LightComponent, LightSettings, LightType, PickedComponent, RenderSurface,
        StaticMesh,
    },
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

        let (shader, root_signature) =
            renderer.prepare_vs_ps(String::from("crate://renderer/shaders/shader.hlsl"));

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

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, Option<&PickedComponent>)],
        camera: &CameraComponent,
        lights: &[(&Transform, &LightComponent)],
        light_settings: &LightSettings,
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

        let (view_matrix, projection_matrix) = camera.build_view_projection(
            render_surface.extents().width() as f32,
            render_surface.extents().height() as f32,
        );
        let transient_allocator = render_context.transient_buffer_allocator();

        const NUM_LIGHTS: usize = 8;
        const DIRECTIONAL_LIGHT_SIZE: usize = 32;
        const OMNIDIRECTIONAL_LIGHT_SIZE: usize = 32;
        const SPOTLIGHT_SIZE: usize = 32;

        // Lights
        let mut directional_lights_data =
            Vec::<f32>::with_capacity(DIRECTIONAL_LIGHT_SIZE * NUM_LIGHTS);
        let mut omnidirectional_lights_data =
            Vec::<f32>::with_capacity(OMNIDIRECTIONAL_LIGHT_SIZE * NUM_LIGHTS);
        let mut spotlights_data = Vec::<f32>::with_capacity(SPOTLIGHT_SIZE * NUM_LIGHTS);
        let mut num_directional_lights = 0;
        let mut num_omnidirectional_lights = 0;
        let mut num_spotlights = 0;
        for (transform, light) in lights {
            if !light.enabled {
                continue;
            }
            match light.light_type {
                LightType::Directional { direction } => {
                    let direction_in_view = view_matrix.mul_vec4(direction.extend(0.0));

                    directional_lights_data.push(direction_in_view.x);
                    directional_lights_data.push(direction_in_view.y);
                    directional_lights_data.push(direction_in_view.z);
                    directional_lights_data.push(light.radiance);
                    directional_lights_data.push(light.color.0);
                    directional_lights_data.push(light.color.1);
                    directional_lights_data.push(light.color.2);
                    num_directional_lights += 1;
                    unsafe {
                        directional_lights_data
                            .set_len(DIRECTIONAL_LIGHT_SIZE / 4 * num_directional_lights as usize);
                    }
                }
                LightType::Omnidirectional => {
                    let transform_in_view = view_matrix.mul_vec4(transform.translation.extend(1.0));

                    omnidirectional_lights_data.push(transform_in_view.x);
                    omnidirectional_lights_data.push(transform_in_view.y);
                    omnidirectional_lights_data.push(transform_in_view.z);
                    omnidirectional_lights_data.push(light.radiance);
                    omnidirectional_lights_data.push(light.color.0);
                    omnidirectional_lights_data.push(light.color.1);
                    omnidirectional_lights_data.push(light.color.2);
                    num_omnidirectional_lights += 1;
                    unsafe {
                        omnidirectional_lights_data.set_len(
                            OMNIDIRECTIONAL_LIGHT_SIZE / 4 * num_omnidirectional_lights as usize,
                        );
                    }
                }
                LightType::Spotlight {
                    direction,
                    cone_angle,
                } => {
                    let transform_in_view = view_matrix.mul_vec4(transform.translation.extend(1.0));
                    let direction_in_view = view_matrix.mul_vec4(direction.extend(0.0));

                    spotlights_data.push(transform_in_view.x);
                    spotlights_data.push(transform_in_view.y);
                    spotlights_data.push(transform_in_view.z);
                    spotlights_data.push(light.radiance);
                    spotlights_data.push(direction_in_view.x);
                    spotlights_data.push(direction_in_view.y);
                    spotlights_data.push(direction_in_view.z);
                    spotlights_data.push(cone_angle);
                    spotlights_data.push(light.color.0);
                    spotlights_data.push(light.color.1);
                    spotlights_data.push(light.color.2);
                    num_spotlights += 1;
                    unsafe {
                        spotlights_data.set_len(SPOTLIGHT_SIZE / 4 * num_spotlights as usize);
                    }
                }
            }
        }
        unsafe {
            directional_lights_data.set_len(DIRECTIONAL_LIGHT_SIZE / 4 * NUM_LIGHTS);
            omnidirectional_lights_data.set_len(OMNIDIRECTIONAL_LIGHT_SIZE / 4 * NUM_LIGHTS);
            spotlights_data.set_len(SPOTLIGHT_SIZE / 4 * NUM_LIGHTS);
        }

        let directional_lights_buffer_view = transient_allocator
            .copy_data_slice(&directional_lights_data, ResourceUsage::AS_SHADER_RESOURCE)
            .structured_buffer_view(DIRECTIONAL_LIGHT_SIZE as u64, true);

        let omnidirectional_lights_buffer_view = transient_allocator
            .copy_data_slice(
                &omnidirectional_lights_data,
                ResourceUsage::AS_SHADER_RESOURCE,
            )
            .structured_buffer_view(OMNIDIRECTIONAL_LIGHT_SIZE as u64, true);

        let spotlights_buffer_view = transient_allocator
            .copy_data_slice(&spotlights_data, ResourceUsage::AS_SHADER_RESOURCE)
            .structured_buffer_view(SPOTLIGHT_SIZE as u64, true);

        for (_index, (static_mesh, picked_component)) in static_meshes.iter().enumerate() {
            let color: (f32, f32, f32, f32) = (
                f32::from(static_mesh.color.r) / 255.0f32,
                f32::from(static_mesh.color.g) / 255.0f32,
                f32::from(static_mesh.color.b) / 255.0f32,
                f32::from(static_mesh.color.a) / 255.0f32,
            );

            let mut constant_data = [0.0; 45];

            view_matrix.write_cols_to_slice(&mut constant_data[0..]);
            projection_matrix.write_cols_to_slice(&mut constant_data[16..]);
            constant_data[32] = color.0;
            constant_data[33] = color.1;
            constant_data[34] = color.2;
            constant_data[35] = 1.0;
            constant_data[36] = f32::from_bits(num_directional_lights);
            constant_data[37] = f32::from_bits(num_omnidirectional_lights);
            constant_data[38] = f32::from_bits(num_spotlights);
            constant_data[39] = f32::from_bits(light_settings.diffuse as u32);
            constant_data[40] = f32::from_bits(light_settings.specular as u32);
            constant_data[41] = light_settings.specular_reflection;
            constant_data[42] = light_settings.diffuse_reflection;
            constant_data[43] = light_settings.ambient_reflection;
            constant_data[44] = light_settings.shininess;

            let sub_allocation =
                transient_allocator.copy_data_slice(&constant_data, ResourceUsage::AS_CONST_BUFFER);

            let const_buffer_view = sub_allocation.const_buffer_view();

            let mut descriptor_set_writer =
                render_context.alloc_descriptor_set(descriptor_set_layout);

            descriptor_set_writer
                .set_descriptors_by_name(
                    "const_data",
                    &[DescriptorRef::BufferView(&const_buffer_view)],
                )
                .unwrap();

            descriptor_set_writer
                .set_descriptors_by_name(
                    "directional_lights",
                    &[DescriptorRef::BufferView(&directional_lights_buffer_view)],
                )
                .unwrap();

            let omnidirectional_lights_buffer_view = render_context
                .renderer()
                .omnidirectional_lights_structured_view();
            descriptor_set_writer
                .set_descriptors_by_name(
                    "omnidirectional_lights",
                    &[DescriptorRef::BufferView(
                        &omnidirectional_lights_buffer_view,
                    )],
                )
                .unwrap();

            descriptor_set_writer
                .set_descriptors_by_name(
                    "spotlights",
                    &[DescriptorRef::BufferView(&spotlights_buffer_view)],
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
            push_constant_data[0] = static_mesh.vertex_offset;
            push_constant_data[1] = static_mesh.world_offset;
            push_constant_data[2] = if picked_component.is_some() { 1 } else { 0 };

            cmd_buffer.push_constants(&self.root_signature, &push_constant_data);

            cmd_buffer.draw(static_mesh.num_verticies, 0);

            /*/ WIP
            {
                let mut pipeline_data =
                    cgen::pipeline_layout::TmpPipelineLayout::new(&self.pipeline);
                render_context.populate_pipeline_data(&mut pipeline_data);
                pipeline_data.set_push_constant(&cgen::cgen_type::PushConstantData {
                    color: Default::default(),
                });
                cmd_buffer.draw_with_data(&pipeline_data, static_mesh_component.num_verticies, 0);
            }
            */
        }

        cmd_buffer.end_render_pass();
    }
}
