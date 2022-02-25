use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Res, ResMut};
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
    labels::RenderStage,
    resources::{PipelineHandle, PipelineManager, UnifiedStaticBuffer, UniformGPUDataUpdater},
    RenderContext, Renderer,
};

use super::{RenderElement, RenderLayer, RenderStateSet};

#[derive(Default)]
pub struct MeshRendererPlugin {}

pub(crate) enum DefaultLayers {
    Opaque = 0,
}

impl Plugin for MeshRendererPlugin {
    fn build(&self, app: &mut App) {
        //
        // Stage Startup
        //
        app.add_startup_system(init_default);

        //
        // Stage Prepare
        //
        app.add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_default(
    pipeline_manager: Res<'_, PipelineManager>,
    mut mesh_renderer: ResMut<'_, MeshRenderer>,
) {
    mesh_renderer.initialize(&pipeline_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn prepare(renderer: Res<'_, Renderer>, mut mesh_renderer: ResMut<'_, MeshRenderer>) {
    mesh_renderer.prepare(&renderer);
}

pub struct MeshRenderer {
    default_layers: Vec<RenderLayer>,
    indirect_arg_buffer: Option<Buffer>,
    count_buffer: Option<Buffer>,
    tmp_batch_idx: u32,
}

impl MeshRenderer {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            default_layers: vec![RenderLayer::new(static_buffer, true)],
            indirect_arg_buffer: None,
            count_buffer: None,
            tmp_batch_idx: 0,
        }
    }

    pub fn initialize(&mut self, pipeline_manager: &PipelineManager) {
        let tmp_pipeline_handle = build_temp_pso(pipeline_manager);

        self.tmp_batch_idx = self.default_layers[DefaultLayers::Opaque as usize]
            .register_state_set(&RenderStateSet {
                pipeline_handle: tmp_pipeline_handle,
            });
    }

    pub fn register_material(&mut self, material_idx: u32) {
        for layer in &mut self.default_layers {
            layer.register_material(material_idx, self.tmp_batch_idx);
        }
    }

    pub fn register_element(&mut self, material_idx: u32, element: &RenderElement) {
        for layer in &mut self.default_layers {
            layer.register_element(material_idx, element);
        }
    }

    pub fn unregister_element(&mut self, material_idx: u32, gpu_instance_id: u32) {
        for layer in &mut self.default_layers {
            layer.unregister_element(material_idx, gpu_instance_id);
        }
    }

    pub fn prepare(&mut self, renderer: &Renderer) {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        let mut count_buffer_size: u64 = 0;
        let mut indirect_arg_buffer_size = 0;

        for layer in &mut self.default_layers {
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
            indirect_arg_buffer_size,
        );
    }

    pub fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        layer_id: usize,
    ) {
        self.default_layers[layer_id].draw(
            render_context,
            cmd_buffer,
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
