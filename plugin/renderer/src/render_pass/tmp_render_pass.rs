#![allow(unsafe_code)]

use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DescriptorRef, Format,
    GraphicsPipelineDef, LoadOp, Pipeline, PipelineType, PrimitiveTopology, RasterizerState,
    ResourceState, ResourceUsage, RootSignature, SampleCount, StencilOp, StoreOp, VertexLayout,
};

use crate::{
    components::{
        CameraComponent, PickedComponent, RenderSurface, StaticMesh,
    },
    hl_gfx_api::HLCommandBuffer,
    lighting::LightingManager,
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
        lighting_manager: &LightingManager,
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

        let mut constant_data = Vec::with_capacity(32);
        unsafe {
            constant_data.set_len(32);
        }
        view_matrix.write_cols_to_slice(&mut constant_data[0..]);
        projection_matrix.write_cols_to_slice(&mut constant_data[16..]);

        let camera_buffer_view = transient_allocator
            .copy_data_slice(&constant_data, ResourceUsage::AS_CONST_BUFFER)
            .const_buffer_view();

        let lighting_manager_view = transient_allocator
            .copy_data_slice(&lighting_manager.gpu_data(), ResourceUsage::AS_CONST_BUFFER)
            .const_buffer_view();
            
        for (_index, (static_mesh, picked_component)) in static_meshes.iter().enumerate() {

            let mut descriptor_set_writer =
                render_context.alloc_descriptor_set(descriptor_set_layout);

            descriptor_set_writer
                .set_descriptors_by_name(
                    "camera",
                    &[DescriptorRef::BufferView(&camera_buffer_view)],
                )
                .unwrap();

            descriptor_set_writer
                .set_descriptors_by_name(
                    "lighting_manager",
                    &[DescriptorRef::BufferView(&lighting_manager_view)],
                )
                .unwrap();

            let directional_lights_buffer_view = render_context
                .renderer()
                .directional_lights_data_structured_buffer_view();
            descriptor_set_writer
                .set_descriptors_by_name(
                    "directional_lights",
                    &[DescriptorRef::BufferView(&directional_lights_buffer_view)],
                )
                .unwrap();

            let omnidirectional_lights_buffer_view = render_context
                .renderer()
                .omnidirectional_lights_data_structured_buffer_view();
            descriptor_set_writer
                .set_descriptors_by_name(
                    "omnidirectional_lights",
                    &[DescriptorRef::BufferView(
                        &omnidirectional_lights_buffer_view,
                    )],
                )
                .unwrap();

            let spotlights_buffer_view = render_context
                .renderer()
                .spotlights_data_structured_buffer_view();
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

            let color: (f32, f32, f32, f32) = (
                f32::from(static_mesh.color.r) / 255.0f32,
                f32::from(static_mesh.color.g) / 255.0f32,
                f32::from(static_mesh.color.b) / 255.0f32,
                f32::from(static_mesh.color.a) / 255.0f32,
            );

            let mut push_constant_data = [0; 8];
            push_constant_data[0] = static_mesh.vertex_offset;
            push_constant_data[1] = static_mesh.world_offset;
            push_constant_data[2] = if picked_component.is_some() { 1 } else { 0 };
            push_constant_data[3] = 0; // padding
            push_constant_data[4] = color.0.to_bits();
            push_constant_data[5] = color.1.to_bits();
            push_constant_data[6] = color.2.to_bits();
            push_constant_data[7] = color.3.to_bits();

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
