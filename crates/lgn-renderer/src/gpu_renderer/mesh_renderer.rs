use lgn_graphics_api::{
    BlendState, Buffer, BufferDef, CompareOp, DepthState, DeviceContext, Format,
    GraphicsPipelineDef, PrimitiveTopology, RasterizerState, ResourceCreation, ResourceUsage,
    SampleCount, StencilOp, VertexAttributeRate, VertexLayout, VertexLayoutAttribute,
    VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;

use crate::{
    cgen,
    hl_gfx_api::HLCommandBuffer,
    resources::{PipelineHandle, PipelineManager, UnifiedStaticBuffer, UniformGPUDataUpdater},
    Renderer,
};

use super::RenderLayer;

enum DefaultLayers {
    Opaque = 0,
}

pub struct MeshRenderer {
    layers: Vec<RenderLayer>,
    tmp_pipeline_handle: PipelineHandle,
    indirect_arg_buffer: Option<Buffer>,
    count_buffer: Option<Buffer>,
}

impl MeshRenderer {
    pub fn new(static_buffer: &UnifiedStaticBuffer, pipeline_manager: &PipelineManager) -> Self {
        let opaque_layer = RenderLayer::new(static_buffer, false);

        Self {
            layers: vec![opaque_layer],
            tmp_pipeline_handle: build_temp_pso(pipeline_manager),
            indirect_arg_buffer: None,
            count_buffer: None,
        }
    }

    pub fn register_material(&self) {}

    pub fn register_element(&self) {}

    pub fn prepare(&mut self, renderer: &Renderer) {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        let count_buffer_size: u64 = 0;
        let indirect_arg_buffer_size = 0;

        for layer in self.layers {
            layer.aggregate_offsets(&mut updater);

            let (batch_inc, element_inc) = layer.get_arg_buffer_sizes();

            count_buffer_size += batch_inc;
            indirect_arg_buffer_size += element_inc;
        }

        renderer.add_update_job_block(updater.job_blocks());

        create_or_replace_buffer(
            renderer.device_context(),
            &mut self.count_buffer,
            count_buffer_size,
        );

        create_or_replace_buffer(
            renderer.device_context(),
            &mut self.indirect_arg_buffer,
            count_buffer_size,
        );
    }

    pub fn cull() {}

    pub fn draw(
        &self,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        pipeline_manager: &PipelineManager,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
        layer_id: usize,
    ) {
        self.layers[layer_id].draw(
            cmd_buffer,
            pipeline_manager,
            self.indirect_arg_buffer.as_ref(),
            self.count_buffer.as_ref(),
        );
    }
}

fn create_or_replace_buffer(
    device_context: &DeviceContext,
    buffer: &mut Option<Buffer>,
    required_size: u64,
) {
    if let Some(count_buffer) = buffer {
        if count_buffer.definition().size < required_size {
            *buffer = None;
        }
    }

    if buffer.is_none() {
        let buffer_def = BufferDef {
            size: required_size,
            usage_flags: ResourceUsage::AS_INDIRECT_BUFFER,
            creation_flags: ResourceCreation::empty(),
        };

        *buffer = Some(device_context.create_buffer(&buffer_def));
    }
}

fn build_temp_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::ShaderPipelineLayout::root_signature();

    let mut vertex_layout = VertexLayout::default();
    vertex_layout.attributes[0] = Some(VertexLayoutAttribute {
        format: Format::R32_UINT,
        buffer_index: 0,
        location: 0,
        byte_offset: 0,
    });
    vertex_layout.buffers[0] = Some(VertexLayoutBuffer {
        stride: 4,
        rate: VertexAttributeRate::Instance,
    });

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

    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(
            cgen::shader::default_shader::ID,
            cgen::shader::default_shader::NONE,
        ),
        move |device_context, shader| {
            device_context
                .create_graphics_pipeline(&GraphicsPipelineDef {
                    shader,
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
                .unwrap()
        },
    )
}
