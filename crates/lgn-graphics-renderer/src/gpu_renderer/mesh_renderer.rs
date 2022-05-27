use lgn_app::{App, CoreStage};
use lgn_core::Handle;
use lgn_ecs::{
    prelude::{Res, ResMut},
    schedule::{ParallelSystemDescriptorCoercion, SystemLabel},
};
use lgn_embedded_fs::embedded_watched_file;
use lgn_graphics_api::{
    BarrierQueueTransition, BlendState, Buffer, BufferBarrier, BufferCreateFlags, BufferDef,
    BufferView, BufferViewDef, CommandBuffer, CompareOp, ComputePipelineDef, CullMode, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, DeviceContext, Format,
    GraphicsPipelineDef, LoadOp, MemoryUsage, PrimitiveTopology, RasterizerState, ResourceState,
    ResourceUsage, SampleCount, StencilOp, StoreOp, TransientBufferView, VertexAttributeRate,
    VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::{
    cgen::{
        self,
        cgen_type::{
            CullingDebugData, CullingEfficiencyStats, CullingOptions, GpuInstanceData,
            RenderPassData,
        },
        shader,
    },
    components::RenderSurface,
    core::BinaryWriter,
    egui::egui_plugin::Egui,
    labels::RenderStage,
    resources::{
        GpuBufferWithReadback, MaterialId, PipelineDef, PipelineHandle, PipelineManager,
        ReadbackBuffer, UnifiedStaticBufferAllocator, UpdateUnifiedStaticBufferCommand,
    },
    RenderContext, Renderer,
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

#[derive(Clone, Copy)]
struct GpuCullingOptions {
    indirect_dispatch: bool,
    gather_perf_stats: bool,
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

pub(crate) struct CullingArgBuffer {
    buffer: Buffer,
    srv_view: BufferView,
    uav_view: BufferView,
}

pub(crate) struct CullingArgBuffers {
    pub(crate) draw_count: Option<CullingArgBuffer>,
    pub(crate) draw_args: Option<CullingArgBuffer>,
    pub(crate) culled_count: Option<CullingArgBuffer>,
    pub(crate) culled_args: Option<CullingArgBuffer>,
    pub(crate) culled_instances: Option<CullingArgBuffer>,
    pub(crate) stats_buffer: GpuBufferWithReadback,
    pub(crate) stats_buffer_readback: Option<Handle<ReadbackBuffer>>,
    pub(crate) culling_debug: Option<CullingArgBuffer>,
    // TMP until shader variations
    pub(crate) tmp_culled_count: Option<CullingArgBuffer>,
    pub(crate) tmp_culled_args: Option<CullingArgBuffer>,
    pub(crate) tmp_culled_instances: Option<CullingArgBuffer>,
}

pub struct MeshRenderer {
    pub(crate) default_layers: Vec<RenderLayer>,

    pub(crate) instance_data_indices: Vec<u32>,
    pub(crate) gpu_instance_data: Vec<GpuInstanceData>,
    pub(crate) depth_count_buffer_size: u64,

    culling_shader_first_pass: Option<PipelineHandle>,
    culling_shader_second_pass: Option<PipelineHandle>,
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
                draw_count: None,
                draw_args: None,
                culled_count: None,
                culled_args: None,
                culled_instances: None,
                culling_debug: None,
                stats_buffer: GpuBufferWithReadback::new(
                    device_context,
                    std::mem::size_of::<CullingEfficiencyStats>() as u64,
                ),
                stats_buffer_readback: None,
                tmp_culled_count: None,
                tmp_culled_args: None,
                tmp_culled_instances: None,
            },
            culling_stats: CullingEfficiencyStats::default(),
            instance_data_indices: vec![],
            gpu_instance_data: vec![],
            depth_count_buffer_size: 0,
            culling_shader_first_pass: None,
            culling_shader_second_pass: None,
            tmp_batch_ids: vec![],
            tmp_pipeline_handles: vec![],
        }
    }

    pub fn initialize_psos(&mut self, pipeline_manager: &PipelineManager) {
        if self.culling_shader_first_pass.is_none() {
            let (first_pass, second_pass) = build_culling_psos(pipeline_manager);
            self.culling_shader_first_pass = Some(first_pass);
            self.culling_shader_second_pass = Some(second_pass);

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
        let mut render_commands = renderer.render_command_builder();
        let device_context = renderer.device_context();

        let mut count_buffer_size: u64 = 0;
        let mut indirect_arg_buffer_size: u64 = 0;
        self.depth_count_buffer_size = 0;

        for (index, layer) in self.default_layers.iter_mut().enumerate() {
            let per_state_offsets =
                layer.aggregate_offsets(&mut count_buffer_size, &mut indirect_arg_buffer_size);
            if index == DefaultLayers::Depth as usize {
                self.depth_count_buffer_size = count_buffer_size;
            }

            if !per_state_offsets.is_empty() {
                let mut binary_writer = BinaryWriter::new();
                binary_writer.write_slice(&per_state_offsets);

                render_commands.push(UpdateUnifiedStaticBufferCommand {
                    src_buffer: binary_writer.take(),
                    dst_offset: layer.state_page.byte_offset(),
                });
            }
        }

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

        if count_buffer_size != 0 {
            create_or_replace_buffer(
                device_context,
                &mut self.culling_buffers.draw_count,
                std::mem::size_of::<u32>() as u64,
                count_buffer_size,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_TRANSFERABLE
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );
        }

        if indirect_arg_buffer_size != 0 {
            create_or_replace_buffer(
                device_context,
                &mut self.culling_buffers.draw_args,
                5 * std::mem::size_of::<u32>() as u64,
                indirect_arg_buffer_size,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );
        }

        create_or_replace_buffer(
            device_context,
            &mut self.culling_buffers.culled_count,
            std::mem::size_of::<u32>() as u64,
            1,
            ResourceUsage::AS_INDIRECT_BUFFER
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS,
            MemoryUsage::GpuOnly,
        );

        create_or_replace_buffer(
            device_context,
            &mut self.culling_buffers.tmp_culled_count,
            std::mem::size_of::<u32>() as u64,
            1,
            ResourceUsage::AS_INDIRECT_BUFFER
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS,
            MemoryUsage::GpuOnly,
        );

        create_or_replace_buffer(
            device_context,
            &mut self.culling_buffers.culled_args,
            std::mem::size_of::<(u32, u32, u32)>() as u64,
            1,
            ResourceUsage::AS_INDIRECT_BUFFER
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS,
            MemoryUsage::GpuOnly,
        );

        create_or_replace_buffer(
            device_context,
            &mut self.culling_buffers.tmp_culled_args,
            std::mem::size_of::<(u32, u32, u32)>() as u64,
            1,
            ResourceUsage::AS_INDIRECT_BUFFER
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS,
            MemoryUsage::GpuOnly,
        );

        if !self.gpu_instance_data.is_empty() {
            create_or_replace_buffer(
                device_context,
                &mut self.culling_buffers.culled_instances,
                std::mem::size_of::<GpuInstanceData>() as u64,
                self.gpu_instance_data.len() as u64,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );

            create_or_replace_buffer(
                device_context,
                &mut self.culling_buffers.culling_debug,
                std::mem::size_of::<CullingDebugData>() as u64,
                self.gpu_instance_data.len() as u64,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );

            create_or_replace_buffer(
                device_context,
                &mut self.culling_buffers.tmp_culled_instances,
                std::mem::size_of::<GpuInstanceData>() as u64,
                self.gpu_instance_data.len() as u64,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn cull(
        &self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        cmd_buffer: &mut CommandBuffer,
        culling_buffers: &CullingArgBuffers,
        culling_options: GpuCullingOptions,
        culling_args: (u32, u32, u32, Vec2),
        input_buffers: (
            TransientBufferView,
            TransientBufferView,
            TransientBufferView,
        ),
    ) {
        cmd_buffer.with_label("Cull", |cmd_buffer| {
            let indirect_dispatch = culling_options.indirect_dispatch;
            let gather_perf_stats = culling_options.gather_perf_stats;

            let draw_count = culling_buffers.draw_count.as_ref().unwrap();
            let draw_args = culling_buffers.draw_args.as_ref().unwrap();
            let (culled_count, culled_args, culled_instances) = if indirect_dispatch {
                (
                    culling_buffers.tmp_culled_count.as_ref().unwrap(),
                    culling_buffers.tmp_culled_args.as_ref().unwrap(),
                    culling_buffers.tmp_culled_instances.as_ref().unwrap(),
                )
            } else {
                (
                    culling_buffers.culled_count.as_ref().unwrap(),
                    culling_buffers.culled_args.as_ref().unwrap(),
                    culling_buffers.culled_instances.as_ref().unwrap(),
                )
            };

            let (dispatch_count, dispatch_args, dispatch_instances) = (
                culling_buffers.culled_count.as_ref().unwrap(),
                culling_buffers.culled_args.as_ref().unwrap(),
                culling_buffers.culled_instances.as_ref().unwrap(),
            );

            let pipeline_handle = if indirect_dispatch {
                self.culling_shader_second_pass.unwrap()
            } else {
                self.culling_shader_first_pass.unwrap()
            };

            let pipeline = render_context
                .pipeline_manager
                .get_pipeline(pipeline_handle)
                .unwrap();
            cmd_buffer.cmd_bind_pipeline(pipeline);

            cmd_buffer.cmd_bind_descriptor_set_handle(
                render_context.frame_descriptor_set().0,
                render_context.frame_descriptor_set().1,
            );

            cmd_buffer.cmd_bind_descriptor_set_handle(
                render_context.view_descriptor_set().0,
                render_context.view_descriptor_set().1,
            );

            let mut culling_descriptor_set = cgen::descriptor_set::CullingDescriptorSet::default();
            culling_descriptor_set.set_draw_count(&draw_count.uav_view);
            culling_descriptor_set.set_draw_args(&draw_args.uav_view);
            culling_descriptor_set.set_culled_count(&culled_count.uav_view);
            culling_descriptor_set.set_culled_args(&culled_args.uav_view);
            culling_descriptor_set.set_culled_instances(&culled_instances.uav_view);
            culling_descriptor_set.set_culling_efficiency(culling_buffers.stats_buffer.rw_view());

            culling_descriptor_set
                .set_culling_debug(&culling_buffers.culling_debug.as_ref().unwrap().uav_view);

            if indirect_dispatch {
                culling_descriptor_set.set_gpu_instance_count(&dispatch_count.srv_view);
                culling_descriptor_set.set_gpu_instance_data(&dispatch_instances.srv_view);
            } else {
                culling_descriptor_set.set_gpu_instance_count(input_buffers.0);
                culling_descriptor_set.set_gpu_instance_data(input_buffers.1);
            }
            culling_descriptor_set.set_render_pass_data(input_buffers.2);

            culling_descriptor_set.set_hzb_texture(render_surface.get_hzb_surface().hzb_srv_view());

            let culling_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
                culling_descriptor_set.descriptor_refs(),
            );

            cmd_buffer.cmd_bind_descriptor_set_handle(
                cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
                culling_descriptor_set_handle,
            );

            cmd_buffer.cmd_resource_barrier(
                &[
                    BufferBarrier {
                        buffer: &draw_count.buffer,
                        src_state: ResourceState::INDIRECT_ARGUMENT,
                        dst_state: ResourceState::COPY_DST,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_count.buffer,
                        src_state: ResourceState::SHADER_RESOURCE,
                        dst_state: ResourceState::COPY_DST,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_args.buffer,
                        src_state: ResourceState::INDIRECT_ARGUMENT,
                        dst_state: ResourceState::COPY_DST,
                        queue_transition: BarrierQueueTransition::None,
                    },
                ],
                &[],
            );

            if indirect_dispatch {
                let depth_count_size =
                    self.depth_count_buffer_size * std::mem::size_of::<u32>() as u64;
                cmd_buffer.cmd_fill_buffer(&draw_count.buffer, 0, depth_count_size, 0);
            } else {
                cmd_buffer.cmd_fill_buffer(
                    &draw_count.buffer,
                    0,
                    draw_count.buffer.definition().size,
                    0,
                );
                cmd_buffer.cmd_fill_buffer(&culled_count.buffer, 0, 4, 0);
                cmd_buffer.cmd_fill_buffer(&culled_args.buffer, 0, 4, 0);
            }
            cmd_buffer.cmd_resource_barrier(
                &[
                    BufferBarrier {
                        buffer: &draw_count.buffer,
                        src_state: ResourceState::COPY_DST,
                        dst_state: ResourceState::UNORDERED_ACCESS,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &draw_args.buffer,
                        src_state: ResourceState::INDIRECT_ARGUMENT,
                        dst_state: ResourceState::UNORDERED_ACCESS,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_count.buffer,
                        src_state: ResourceState::COPY_DST,
                        dst_state: ResourceState::UNORDERED_ACCESS,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_args.buffer,
                        src_state: ResourceState::COPY_DST,
                        dst_state: ResourceState::UNORDERED_ACCESS,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_instances.buffer,
                        src_state: ResourceState::SHADER_RESOURCE,
                        dst_state: ResourceState::UNORDERED_ACCESS,
                        queue_transition: BarrierQueueTransition::None,
                    },
                ],
                &[],
            );

            let mut options = CullingOptions::empty();
            if gather_perf_stats {
                options |= CullingOptions::GATHER_PERF_STATS;
            }

            let mut culling_constant_data = cgen::cgen_type::CullingPushConstantData::default();
            culling_constant_data.set_first_render_pass(culling_args.0.into());
            culling_constant_data.set_num_render_passes(culling_args.1.into());
            culling_constant_data.set_hzb_max_lod(culling_args.2.into());
            culling_constant_data.set_hzb_pixel_extents(culling_args.3.into());
            culling_constant_data.set_options(options);

            cmd_buffer.cmd_push_constant_typed(&culling_constant_data);

            if indirect_dispatch {
                cmd_buffer.cmd_dispatch_indirect(&dispatch_args.buffer, 0);
            } else {
                cmd_buffer.cmd_dispatch((self.gpu_instance_data.len() as u32 + 255) / 256, 1, 1);
            }

            cmd_buffer.cmd_resource_barrier(
                &[
                    BufferBarrier {
                        buffer: &draw_count.buffer,
                        src_state: ResourceState::UNORDERED_ACCESS,
                        dst_state: ResourceState::INDIRECT_ARGUMENT,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &draw_args.buffer,
                        src_state: ResourceState::UNORDERED_ACCESS,
                        dst_state: ResourceState::INDIRECT_ARGUMENT,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_count.buffer,
                        src_state: ResourceState::UNORDERED_ACCESS,
                        dst_state: ResourceState::SHADER_RESOURCE,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_args.buffer,
                        src_state: ResourceState::UNORDERED_ACCESS,
                        dst_state: ResourceState::INDIRECT_ARGUMENT,
                        queue_transition: BarrierQueueTransition::None,
                    },
                    BufferBarrier {
                        buffer: &culled_instances.buffer,
                        src_state: ResourceState::UNORDERED_ACCESS,
                        dst_state: ResourceState::SHADER_RESOURCE,
                        queue_transition: BarrierQueueTransition::None,
                    },
                ],
                &[],
            );
        });
    }

    pub(crate) fn gen_occlusion_and_cull(
        &self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        render_surface: &mut RenderSurface,
        instance_manager: &GpuInstanceManager,
    ) {
        if self.culling_buffers.draw_count.is_none() || self.gpu_instance_data.is_empty() {
            // TODO(vdbdd):  Remove this hack

            cmd_buffer.cmd_begin_render_pass(
                &[],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: render_surface.depth_rt().rtv(),
                    depth_load_op: LoadOp::Clear,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::Store,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                }),
            );

            cmd_buffer.cmd_end_render_pass();

            return;
        }

        let mut render_pass_data: Vec<RenderPassData> = vec![];
        for layer in &self.default_layers {
            let offset_base_va = u32::try_from(layer.offsets_va()).unwrap();

            let mut pass_data = RenderPassData::default();
            pass_data.set_offset_base_va(offset_base_va.into());
            render_pass_data.push(pass_data);
        }

        cmd_buffer.with_label("Gen occlusion and cull", |cmd_buffer| {
            cmd_buffer.cmd_bind_index_buffer(render_context.static_buffer.index_buffer_binding());
            cmd_buffer.cmd_bind_vertex_buffer(0, instance_manager.vertex_buffer_binding());

            let hzb_pixel_extents = render_surface.get_hzb_surface().hzb_pixel_extents();
            let hzb_max_lod = render_surface.get_hzb_surface().hzb_max_lod();

            render_surface.init_hzb_if_needed(render_context, cmd_buffer);

            let gpu_count_allocation = render_context.transient_buffer_allocator.copy_data(
                &(self.gpu_instance_data.len() as u32),
                ResourceUsage::AS_SHADER_RESOURCE,
            );

            let gpu_count_view = gpu_count_allocation
                .to_buffer_view(BufferViewDef::as_structured_buffer_typed::<u32>(1, true));

            let gpu_instance_allocation = render_context
                .transient_buffer_allocator
                .copy_data_slice(&self.gpu_instance_data, ResourceUsage::AS_SHADER_RESOURCE);

            let gpu_instance_view = gpu_instance_allocation.to_buffer_view(
                BufferViewDef::as_structured_buffer_typed::<GpuInstanceData>(
                    self.gpu_instance_data.len() as u64,
                    true,
                ),
            );

            let render_pass_allocation = render_context
                .transient_buffer_allocator
                .copy_data_slice(&render_pass_data, ResourceUsage::AS_SHADER_RESOURCE);

            let render_pass_view =
                render_pass_allocation.to_buffer_view(BufferViewDef::as_structured_buffer_typed::<
                    RenderPassData,
                >(
                    render_pass_data.len() as u64, true
                ));

            self.culling_buffers.stats_buffer.clear_buffer(cmd_buffer);

            // Cull using previous frame Hzb
            self.cull(
                render_context,
                render_surface,
                cmd_buffer,
                &self.culling_buffers,
                GpuCullingOptions {
                    indirect_dispatch: false,
                    gather_perf_stats: true,
                },
                (
                    0,
                    render_pass_data.len() as u32,
                    hzb_max_lod,
                    hzb_pixel_extents,
                ),
                (gpu_count_view, gpu_instance_view, render_pass_view),
            );

            cmd_buffer.cmd_begin_render_pass(
                &[],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: render_surface.depth_rt().rtv(),
                    depth_load_op: LoadOp::Clear,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::Store,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                }),
            );

            // Render initial depth buffer from last frame culling results
            self.draw(render_context, cmd_buffer, DefaultLayers::Depth);

            cmd_buffer.cmd_end_render_pass();

            // Initial Hzb for current frame
            render_surface.generate_hzb(render_context, cmd_buffer);

            // Rebind global vertex buffer after gen Hzb changes it
            cmd_buffer.cmd_bind_vertex_buffer(0, instance_manager.vertex_buffer_binding());

            // Retest elements culled from first pass against new Hzb
            self.cull(
                render_context,
                render_surface,
                cmd_buffer,
                &self.culling_buffers,
                GpuCullingOptions {
                    indirect_dispatch: true,
                    gather_perf_stats: true,
                },
                (
                    0,
                    render_pass_data.len() as u32,
                    hzb_max_lod,
                    hzb_pixel_extents,
                ),
                (gpu_count_view, gpu_instance_view, render_pass_view),
            );

            // Redraw depth istances that passed second cull pass
            cmd_buffer.cmd_begin_render_pass(
                &[],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: render_surface.depth_rt().rtv(),
                    depth_load_op: LoadOp::Load,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::Store,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue::default(),
                }),
            );

            // Render initial depth buffer from last frame culling results
            self.draw(render_context, cmd_buffer, DefaultLayers::Depth);

            cmd_buffer.cmd_end_render_pass();

            // Update Hzb from complete depth buffer
            render_surface.generate_hzb(render_context, cmd_buffer);

            if let Some(readback) = &self.culling_buffers.stats_buffer_readback {
                self.culling_buffers
                    .stats_buffer
                    .copy_buffer_to_readback(cmd_buffer, readback);
            }
        });
    }

    pub(crate) fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        layer_id: DefaultLayers,
    ) {
        let label = format!(
            "Draw layer: {}",
            match &layer_id {
                DefaultLayers::Depth => "Depth",
                DefaultLayers::Opaque => "Opaque",
                DefaultLayers::Picking => "Picking",
            }
        );

        cmd_buffer.with_label(&label, |cmd_buffer| {
            let layer_id_index = layer_id as usize;
            self.default_layers[layer_id_index].draw(
                render_context,
                cmd_buffer,
                self.culling_buffers
                    .draw_args
                    .as_ref()
                    .map(|buffer| &buffer.buffer),
                self.culling_buffers
                    .draw_count
                    .as_ref()
                    .map(|buffer| &buffer.buffer),
            );
        });
    }

    pub(crate) fn end_frame(&mut self) {
        let readback = std::mem::take(&mut self.culling_buffers.stats_buffer_readback);

        if let Some(readback) = readback {
            self.culling_buffers.stats_buffer.end_readback(readback);
        }
    }
}

impl Drop for MeshRenderer {
    fn drop(&mut self) {
        println!("MeshRenderer dropped");
    }
}

fn create_or_replace_buffer(
    device_context: &DeviceContext,
    buffer: &mut Option<CullingArgBuffer>,
    element_size: u64,
    element_count: u64,
    usage_flags: ResourceUsage,
    memory_usage: MemoryUsage,
) {
    let required_size = element_count * element_size;

    if let Some(optional_buffer) = buffer {
        if optional_buffer.buffer.definition().size < required_size {
            *buffer = None;
        }
    }

    if buffer.is_none() {
        let new_buffer = device_context.create_buffer(
            BufferDef {
                size: required_size,
                usage_flags,
                create_flags: BufferCreateFlags::empty(),
                memory_usage,
                always_mapped: false,
            },
            "culling_args",
        );

        let srv_view = new_buffer.create_view(BufferViewDef::as_structured_buffer_with_offset(
            element_count,
            element_size,
            true,
            0,
        ));

        let uav_view = new_buffer.create_view(BufferViewDef::as_structured_buffer_with_offset(
            element_count,
            element_size,
            false,
            0,
        ));

        *buffer = Some(CullingArgBuffer {
            buffer: new_buffer,
            srv_view,
            uav_view,
        });
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

fn build_culling_psos(pipeline_manager: &PipelineManager) -> (PipelineHandle, PipelineHandle) {
    let root_signature = cgen::pipeline_layout::CullingPipelineLayout::root_signature();

    let shader_first_pass = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                shader::culling_shader::ID,
                shader::culling_shader::FIRST_PASS,
            ),
        )
        .unwrap();

    let shader_second_pass = pipeline_manager
        .create_shader(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                shader::culling_shader::ID,
                shader::culling_shader::SECOND_PASS,
            ),
        )
        .unwrap();

    (
        pipeline_manager.register_pipeline(PipelineDef::Compute(ComputePipelineDef {
            shader: shader_first_pass,
            root_signature: root_signature.clone(),
        })),
        pipeline_manager.register_pipeline(PipelineDef::Compute(ComputePipelineDef {
            shader: shader_second_pass,
            root_signature: root_signature.clone(),
        })),
    )
}
