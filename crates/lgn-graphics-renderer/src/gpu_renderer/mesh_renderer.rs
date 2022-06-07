use lgn_app::{App, CoreStage};
use lgn_core::Handle;
use lgn_ecs::{
    prelude::{Res, ResMut},
    schedule::{ParallelSystemDescriptorCoercion, SystemLabel},
};
use lgn_embedded_fs::embedded_watched_file;
use lgn_graphics_api::{
    BlendState, CompareOp, CullMode, DepthState, DeviceContext, Format,
    GraphicsPipelineDef, PrimitiveTopology, RasterizerState, SampleCount, StencilOp,
    VertexAttributeRate, VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;

use crate::{
    cgen::{
        self,
        cgen_type::{CullingEfficiencyStats, GpuInstanceData},
        shader,
    },
    egui::egui_plugin::Egui,
    labels::RenderStage,
    resources::{
        GpuBufferWithReadback, MaterialId, PipelineDef, PipelineHandle, PipelineManager,
        ReadbackBuffer, UnifiedStaticBufferAllocator,
    },
    Renderer,
};

use super::{
    GpuInstanceId, GpuInstanceManager, GpuInstanceManagerLabel, RenderElement, RenderLayer,
    RenderStateSet,
};

embedded_watched_file!(INCLUDE_BRDF, "gpu/include/brdf.hsh");
embedded_watched_file!(INCLUDE_COMMON, "gpu/include/common.hsh");
embedded_watched_file!(
    INCLUDE_FULLSCREEN_TRIANGLE,
    "gpu/include/fullscreen_triangle.hsh"
);
embedded_watched_file!(INCLUDE_MESH, "gpu/include/mesh.hsh");
embedded_watched_file!(INCLUDE_TRANSFORM, "gpu/include/transform.hsh");
embedded_watched_file!(SHADER_SHADER, "gpu/shaders/shader.hlsl");

pub(crate) type RenderLayerId = u32;

#[derive(Clone, Copy)]
pub(crate) struct RenderLayerMask(pub u64);

impl RenderLayerMask {
    #[allow(dead_code)]
    pub fn iter(self) -> RenderLayerIterator {
        RenderLayerIterator::new(self)
    }
}

pub(crate) struct RenderLayerIterator {
    mask: RenderLayerMask,
}

impl RenderLayerIterator {
    pub fn new(mask: RenderLayerMask) -> Self {
        Self { mask }
    }
}

impl Iterator for RenderLayerIterator {
    type Item = RenderLayerId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask.0 == 0 {
            None
        } else {
            let leading_zero = self.mask.0.trailing_zeros();
            self.mask.0 &= !(1 << leading_zero);
            Some(leading_zero)
        }
    }
}

pub(crate) enum DefaultLayers {
    Depth = 0,
    Opaque,
    Picking,
}

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum MeshRendererLabel {
    UpdateDone,
}

impl MeshRenderer {
    pub fn init_ecs(app: &mut App) {
        //
        // Stage PreUpdate
        //
        // TODO(vdbdd): remove asap
        app.add_system_to_stage(CoreStage::PreUpdate, initialize_psos);

        //
        // Stage Prepare
        //

        // TODO(vdbdd): merge those systems

        app.add_system_to_stage(
            RenderStage::Prepare,
            update_render_elements
                .after(GpuInstanceManagerLabel::UpdateDone)
                .label(MeshRendererLabel::UpdateDone),
        );

        app.add_system_to_stage(
            RenderStage::Prepare,
            prepare.after(MeshRendererLabel::UpdateDone),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn initialize_psos(pipeline_manager: Res<'_, PipelineManager>, renderer: Res<'_, Renderer>) {
    let mut mesh_renderer = renderer.render_resources().get_mut::<MeshRenderer>();
    mesh_renderer.initialize_psos(&pipeline_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn update_render_elements(renderer: Res<'_, Renderer>) {
    let mut mesh_renderer = renderer.render_resources().get_mut::<MeshRenderer>();
    let instance_manager = renderer.render_resources().get::<GpuInstanceManager>();
    instance_manager.for_each_removed_gpu_instance_id(|gpu_instance_id| {
        mesh_renderer.unregister_element(*gpu_instance_id);
    });

    instance_manager.for_each_render_element_added(|render_element| {
        mesh_renderer.register_material(render_element.material_id());
        mesh_renderer.register_element(render_element);
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_mesh_renderer(egui: Res<'_, Egui>, renderer: ResMut<'_, Renderer>) {
    // renderer is a ResMut just to avoid concurrent accesses
    let mesh_renderer = renderer.render_resources().get::<MeshRenderer>();

    egui.window("Culling", |ui| {
        ui.label(format!(
            "Total Elements'{}'",
            u32::from(mesh_renderer.culling_stats.total_elements())
        ));
        ui.label(format!(
            "Frustum Visible '{}'",
            u32::from(mesh_renderer.culling_stats.frustum_visible())
        ));
        ui.label(format!(
            "Occlusiion Visible '{}'",
            u32::from(mesh_renderer.culling_stats.occlusion_visible())
        ));
    });
}

#[allow(clippy::needless_pass_by_value)]
fn prepare(renderer: Res<'_, Renderer>) {
    let mut mesh_renderer = renderer.render_resources().get_mut::<MeshRenderer>();
    mesh_renderer.prepare(&renderer);
}

// TMP -- what is public here is because they are used in the render graph
pub(crate) struct CullingArgBuffers {
    pub(crate) stats_buffer: GpuBufferWithReadback,
    pub(crate) stats_buffer_readback: Option<Handle<ReadbackBuffer>>,
}

// TMP -- what is public here is because they are used in the render graph
pub struct MeshRenderer {
    pub(crate) default_layers: Vec<RenderLayer>,

    pub(crate) instance_data_indices: Vec<u32>,
    pub(crate) gpu_instance_data: Vec<GpuInstanceData>,

    pub(crate) culling_buffers: CullingArgBuffers,
    culling_stats: CullingEfficiencyStats,

    tmp_batch_ids: Vec<u32>,
    tmp_pipeline_handles: Vec<PipelineHandle>,
}

impl MeshRenderer {
    pub(crate) fn new(
        device_context: &DeviceContext,
        allocator: &UnifiedStaticBufferAllocator,
    ) -> Self {
        Self {
            default_layers: vec![
                RenderLayer::new(allocator, false),
                RenderLayer::new(allocator, false),
                RenderLayer::new(allocator, false),
            ],
            culling_buffers: CullingArgBuffers {
                stats_buffer: GpuBufferWithReadback::new(
                    device_context,
                    std::mem::size_of::<CullingEfficiencyStats>() as u64,
                ),
                stats_buffer_readback: None,
            },
            culling_stats: CullingEfficiencyStats::default(),
            instance_data_indices: vec![],
            gpu_instance_data: vec![],
            tmp_batch_ids: vec![],
            tmp_pipeline_handles: vec![],
        }
    }

    pub fn initialize_psos(&mut self, pipeline_manager: &PipelineManager) {
        if self.tmp_pipeline_handles.is_empty() {
            let pipeline_handle = build_depth_pso(pipeline_manager);
            self.tmp_batch_ids.push(
                self.default_layers[DefaultLayers::Depth as usize]
                    .register_state_set(&RenderStateSet { pipeline_handle }),
            );
            self.tmp_pipeline_handles.push(pipeline_handle);

            let need_depth_write =
                !self.default_layers[DefaultLayers::Opaque as usize].gpu_culling_enabled();
            let pipeline_handle = build_temp_pso(pipeline_manager, need_depth_write);
            self.tmp_batch_ids.push(
                self.default_layers[DefaultLayers::Opaque as usize]
                    .register_state_set(&RenderStateSet { pipeline_handle }),
            );
            self.tmp_pipeline_handles.push(pipeline_handle);

            let pipeline_handle = build_picking_pso(pipeline_manager);
            self.tmp_batch_ids.push(
                self.default_layers[DefaultLayers::Picking as usize]
                    .register_state_set(&RenderStateSet { pipeline_handle }),
            );
            self.tmp_pipeline_handles.push(pipeline_handle);
        }
    }

    pub(crate) fn get_tmp_pso_handle(&self, layer_id: usize) -> PipelineHandle {
        self.tmp_pipeline_handles[layer_id]
    }

    fn register_material(&mut self, _material_id: MaterialId) {
        for (index, layer) in &mut self.default_layers.iter_mut().enumerate() {
            layer.register_state(0, self.tmp_batch_ids[index]);
        }
    }

    fn register_element(&mut self, element: &RenderElement) {
        let new_index = self.gpu_instance_data.len() as u32;
        let gpu_instance_index = element.gpu_instance_id().index();
        if gpu_instance_index >= self.instance_data_indices.len() as u32 {
            self.instance_data_indices
                .resize(gpu_instance_index as usize + 1, u32::MAX);
        }
        assert!(self.instance_data_indices[gpu_instance_index as usize] == u32::MAX);
        self.instance_data_indices[gpu_instance_index as usize] = new_index;

        let mut instance_data = GpuInstanceData::default();
        instance_data.set_gpu_instance_id(gpu_instance_index.into());
        instance_data.set_state_id(0.into());

        for layer in &mut self.default_layers {
            layer.register_element(0, element);
        }

        self.gpu_instance_data.push(instance_data);

        self.invariant();
    }

    fn unregister_element(&mut self, gpu_instance_id: GpuInstanceId) {
        let gpu_instance_index = gpu_instance_id.index();
        let removed_index = self.instance_data_indices[gpu_instance_index as usize] as usize;
        assert!(removed_index as u32 != u32::MAX);
        self.instance_data_indices[gpu_instance_index as usize] = u32::MAX;

        let removed_instance = self.gpu_instance_data.swap_remove(removed_index as usize);
        let removed_instance_id: u32 = removed_instance.gpu_instance_id().into();
        assert!(gpu_instance_index == removed_instance_id);

        if removed_index < self.gpu_instance_data.len() {
            let moved_instance_id: u32 = self.gpu_instance_data[removed_index as usize]
                .gpu_instance_id()
                .into();
            self.instance_data_indices[moved_instance_id as usize] = removed_index as u32;
        }

        for layer in &mut self.default_layers {
            layer.unregister_element(removed_instance.state_id().into(), gpu_instance_id);
        }

        self.invariant();
    }

    fn invariant(&self) {
        for (instance_idx, slot_idx) in self.instance_data_indices.iter().enumerate() {
            if *slot_idx != u32::MAX {
                let gpu_instance_data = &self.gpu_instance_data[*slot_idx as usize];
                assert!(gpu_instance_data.gpu_instance_id() == (instance_idx as u32).into());
            }
        }
    }

    fn prepare(&mut self, renderer: &Renderer) {
        let device_context = renderer.device_context();

        let readback = self
            .culling_buffers
            .stats_buffer
            .begin_readback(device_context);

        readback.read_gpu_data(
            0,
            usize::MAX,
            u64::MAX,
            |data: &[CullingEfficiencyStats]| {
                self.culling_stats = data[0];
            },
        );
        self.culling_buffers.stats_buffer_readback = Some(readback);
    }

    pub(crate) fn end_frame(&mut self) {
        let readback = std::mem::take(&mut self.culling_buffers.stats_buffer_readback);

        if let Some(readback) = readback {
            self.culling_buffers.stats_buffer.end_readback(readback);
        }
    }
}

fn build_depth_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::DepthPipelineLayout::root_signature();

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
        depth_compare_op: CompareOp::GreaterOrEqual,
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

    let rasterizer_state = RasterizerState {
        cull_mode: CullMode::Back,
        ..RasterizerState::default()
    };

    let shader = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::depth_shader::ID,
                cgen::shader::depth_shader::NONE,
            ),
        )
        .unwrap();
    pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
        shader,
        root_signature: root_signature.clone(),
        vertex_layout,
        blend_state: BlendState::default_alpha_disabled(),
        depth_state,
        rasterizer_state,
        color_formats: vec![],
        sample_count: SampleCount::SampleCount1,
        depth_stencil_format: Some(Format::D32_SFLOAT),
        primitive_topology: PrimitiveTopology::TriangleList,
    }))
}

fn build_temp_pso(pipeline_manager: &PipelineManager, need_depth_write: bool) -> PipelineHandle {
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
        depth_write_enable: need_depth_write,
        depth_compare_op: if need_depth_write {
            CompareOp::GreaterOrEqual
        } else {
            CompareOp::Equal
        },
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

    let rasterizer_state = RasterizerState {
        cull_mode: CullMode::Back,
        ..RasterizerState::default()
    };

    let shader = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                cgen::shader::default_shader::ID,
                cgen::shader::default_shader::NONE,
            ),
        )
        .unwrap();
    pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
        shader,
        root_signature: root_signature.clone(),
        vertex_layout,
        blend_state: BlendState::default_alpha_disabled(),
        depth_state,
        rasterizer_state,
        color_formats: vec![Format::R16G16B16A16_SFLOAT],
        sample_count: SampleCount::SampleCount1,
        depth_stencil_format: Some(Format::D32_SFLOAT),
        primitive_topology: PrimitiveTopology::TriangleList,
    }))
}

fn build_picking_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::PickingPipelineLayout::root_signature();

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
        depth_test_enable: false,
        depth_write_enable: false,
        depth_compare_op: CompareOp::default(),
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

    let shader = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(shader::picking_shader::ID, shader::picking_shader::NONE),
        )
        .unwrap();
    pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
        shader,
        root_signature: root_signature.clone(),
        vertex_layout,
        blend_state: BlendState::default_alpha_disabled(),
        depth_state,
        rasterizer_state: RasterizerState::default(),
        color_formats: vec![Format::R16G16B16A16_SFLOAT],
        sample_count: SampleCount::SampleCount1,
        depth_stencil_format: None,
        primitive_topology: PrimitiveTopology::TriangleList,
    }))
}
