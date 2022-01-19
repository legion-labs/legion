#![allow(unsafe_code)]

use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, Format, GraphicsPipelineDef, LoadOp,
    Pipeline, PrimitiveTopology, RasterizerState, ResourceState, SampleCount, StencilOp, StoreOp,
    VertexLayout,
};
use lgn_math::Vec4;
use lgn_tracing::span_fn;

use crate::{
    cgen,
    components::{PickedComponent, RenderSurface, StaticMesh},
    hl_gfx_api::HLCommandBuffer,
    RenderContext, Renderer,
};

pub struct TmpRenderPass {
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        let root_signature = cgen::pipeline_layout::ShaderPipelineLayout::root_signature();

        let shader =
            renderer.prepare_vs_ps_no_rs(String::from("crate://renderer/shaders/shader.hlsl"));

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
                root_signature,
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

    #[span_fn]
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, Option<&PickedComponent>)],
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
        cmd_buffer.bind_descriptor_set_handle3(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle3(render_context.view_descriptor_set_handle());

        // let descriptor_set_layout = &self
        //     .pipeline
        //     .root_signature()
        //     .definition()
        //     .descriptor_set_layouts[0];

        // let transient_allocator = render_context.transient_buffer_allocator();

        // let view_data = camera.tmp_build_view_data(
        //     render_surface.extents().width() as f32,
        //     render_surface.extents().height() as f32,
        //     0.0,
        //     0.0,
        //     0.0,
        //     0.0,
        // );

        // let camera_buffer_view = transient_allocator
        //     .copy_data(&view_data, ResourceUsage::AS_CONST_BUFFER)
        //     .const_buffer_view();

        // let lighting_manager_view = transient_allocator
        //     .copy_data(&lighting_manager.gpu_data(), ResourceUsage::AS_CONST_BUFFER)
        //     .const_buffer_view();

            // let mut descriptor_set_writer =
            //  render_context.alloc_descriptor_set(descriptor_set_layout);

            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "view_data",
            //         &[DescriptorRef::BufferView(&camera_buffer_view)],
            //     )
            //     .unwrap();

            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "lighting_data",
            //         &[DescriptorRef::BufferView(&lighting_manager_view)],
            //     )
            //     .unwrap();

            // let directional_lights_buffer_view = render_context
            //     .renderer()
            //     .directional_lights_data_structured_buffer_view();
            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "directional_lights",
            //         &[DescriptorRef::BufferView(&directional_lights_buffer_view)],
            //     )
            //     .unwrap();

            // let omnidirectional_lights_buffer_view = render_context
            //     .renderer()
            //     .omnidirectional_lights_data_structured_buffer_view();
            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "omnidirectional_lights",
            //         &[DescriptorRef::BufferView(
            //             &omnidirectional_lights_buffer_view,
            //         )],
            //     )
            //     .unwrap();

            // let spotlights_buffer_view = render_context
            //     .renderer()
            //     .spotlights_data_structured_buffer_view();
            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "spotlights",
            //         &[DescriptorRef::BufferView(&spotlights_buffer_view)],
            //     )
            //     .unwrap();

            // let static_buffer_ro_view = render_context.renderer().static_buffer_ro_view();
            // descriptor_set_writer
            //     .set_descriptors_by_name(
            //         "static_buffer",
            //         &[DescriptorRef::BufferView(&static_buffer_ro_view)],
            //     )
            //     .unwrap();

            // let descriptor_set_handle =
            //  descriptor_set_writer.flush(render_context.renderer().device_context());

            // cmd_buffer.bind_descriptor_set_handle(
            //     PipelineType::Graphics,
            //     &self.root_signature,
            //     descriptor_set_layout.definition().frequency,
            //     descriptor_set_handle,
            // );

        for (_index, (static_mesh, picked_component)) in static_meshes.iter().enumerate() {
            let color: (f32, f32, f32, f32) = (
                f32::from(static_mesh.color.r) / 255.0f32,
                f32::from(static_mesh.color.g) / 255.0f32,
                f32::from(static_mesh.color.b) / 255.0f32,
                f32::from(static_mesh.color.a) / 255.0f32,
            );

            let mut push_constant_data = cgen::cgen_type::InstancePushConstantData::default();

            push_constant_data.set_vertex_offset(static_mesh.vertex_offset.into());
            push_constant_data.set_world_offset(static_mesh.world_offset.into());
            push_constant_data.set_is_picked(if picked_component.is_some() { 1 } else { 0 }.into());
            push_constant_data.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());

            cmd_buffer.push_constant(self.pipeline.root_signature(), &push_constant_data);

            cmd_buffer.draw(static_mesh.num_vertices, 0);
        }

        cmd_buffer.end_render_pass();
    }
}
