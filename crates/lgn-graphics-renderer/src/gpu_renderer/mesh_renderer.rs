use lgn_app::{App, CoreStage, EventReader};
use lgn_core::Handle;
use lgn_ecs::prelude::{Res, ResMut};
use lgn_embedded_fs::embedded_watched_file;
use lgn_graphics_api::{
    BarrierQueueTransition, BlendState, Buffer, BufferBarrier, BufferDef, BufferView,
    BufferViewDef, CompareOp, ComputePipelineDef, CullMode, DepthState, DepthStencilClearValue,
    DepthStencilRenderTargetBinding, DeviceContext, Format, GraphicsPipelineDef, LoadOp,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, PrimitiveTopology, RasterizerState,
    ResourceCreation, ResourceState, ResourceUsage, SampleCount, StencilOp, StoreOp,
    VertexAttributeRate, VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::{
    cgen::{
        self,
        cgen_type::{
            CullingDebugData, CullingEfficiancyStats, CullingOptions, GpuInstanceData,
            RenderPassData,
        },
        shader,
    },
    components::RenderSurface,
    egui::egui_plugin::Egui,
    hl_gfx_api::HLCommandBuffer,
    labels::RenderStage,
    resources::{
        GpuBufferWithReadback, PipelineHandle, PipelineManager, ReadbackBuffer,
        UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
    },
    RenderContext, Renderer,
};

use super::{GpuInstanceEvent, GpuInstanceManager, RenderElement, RenderLayer, RenderStateSet};

embedded_watched_file!(INCLUDE_BRDF, "gpu/include/brdf.hsh");
embedded_watched_file!(INCLUDE_COMMON, "gpu/include/common.hsh");
embedded_watched_file!(INCLUDE_MESH, "gpu/include/mesh.hsh");
embedded_watched_file!(SHADER_SHADER, "gpu/shaders/shader.hlsl");
struct IndirectDispatch(bool);
struct GatherPerfStats(bool);

pub(crate) enum DefaultLayers {
    Depth = 0,
    Opaque,
    Picking,
}

impl MeshRenderer {
    pub fn init_ecs(app: &mut App) {
        //
        // Events
        //
        app.add_event::<GpuInstanceEvent>();

        //
        // Stage PreUpdate
        //
        app.add_system_to_stage(CoreStage::PreUpdate, initialize_psos);

        //
        // Stage Update
        //
        app.add_system_to_stage(CoreStage::Update, update_render_elements);

        //
        // Stage Prepare
        //
        app.add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn initialize_psos(
    pipeline_manager: Res<'_, PipelineManager>,
    mut mesh_renderer: ResMut<'_, MeshRenderer>,
) {
    mesh_renderer.initialize_psos(&pipeline_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn update_render_elements(
    mut mesh_renderer: ResMut<'_, MeshRenderer>,
    mut event_reader: EventReader<'_, '_, GpuInstanceEvent>,
) {
    for event in event_reader.iter() {
        match event {
            GpuInstanceEvent::Added(added_instances) => {
                for instance in added_instances {
                    mesh_renderer.register_material(instance.0);
                    mesh_renderer.register_element(instance.0, &instance.1);
                }
            }
            GpuInstanceEvent::Removed(removed_instances) => {
                for instance in removed_instances {
                    mesh_renderer.unregister_element(*instance);
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_mesh_renderer(egui_ctx: Res<'_, Egui>, mesh_renderer: Res<'_, MeshRenderer>) {
    egui::Window::new("Culling").show(&egui_ctx.ctx, |ui| {
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
fn prepare(renderer: Res<'_, Renderer>, mut mesh_renderer: ResMut<'_, MeshRenderer>) {
    mesh_renderer.prepare(&renderer);
}

struct CullingArgBuffer {
    buffer: Buffer,
    srv_view: BufferView,
    uav_view: BufferView,
    _allocation: MemoryAllocation,
}

struct CullingArgBuffers {
    draw_count: Option<CullingArgBuffer>,
    draw_args: Option<CullingArgBuffer>,
    culled_count: Option<CullingArgBuffer>,
    culled_args: Option<CullingArgBuffer>,
    culled_instances: Option<CullingArgBuffer>,
    stats_buffer: GpuBufferWithReadback,
    stats_buufer_readback: Option<Handle<ReadbackBuffer>>,
    culling_debug: Option<CullingArgBuffer>,
    // TMP until shader variations
    tmp_culled_count: Option<CullingArgBuffer>,
    tmp_culled_args: Option<CullingArgBuffer>,
    tmp_culled_instances: Option<CullingArgBuffer>,
}

pub struct MeshRenderer {
    default_layers: Vec<RenderLayer>,

    instance_data_idxs: Vec<u32>,
    gpu_instance_data: Vec<GpuInstanceData>,
    depth_count_buffer_count: u64,

    culling_shader_first_pass: Option<PipelineHandle>,
    culling_shader_second_pass: Option<PipelineHandle>,
    culling_buffers: CullingArgBuffers,
    culling_stats: CullingEfficiancyStats,

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
                    std::mem::size_of::<CullingEfficiancyStats>() as u64,
                ),
                stats_buufer_readback: None,
                tmp_culled_count: None,
                tmp_culled_args: None,
                tmp_culled_instances: None,
            },
            culling_stats: CullingEfficiancyStats::default(),
            instance_data_idxs: vec![],
            gpu_instance_data: vec![],
            depth_count_buffer_count: 0,
            culling_shader_first_pass: None,
            culling_shader_second_pass: None,
            tmp_batch_ids: vec![],
            tmp_pipeline_handles: vec![],
        }
    }

    fn initialize_psos(&mut self, pipeline_manager: &PipelineManager) {
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

            let pipeline_handle = build_temp_pso(pipeline_manager);
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

    fn register_material(&mut self, _material_id: u32) {
        for (index, layer) in &mut self.default_layers.iter_mut().enumerate() {
            layer.register_state(0, self.tmp_batch_ids[index]);
        }
    }

    fn register_element(&mut self, _material_id: u32, element: &RenderElement) {
        let new_index = self.gpu_instance_data.len() as u32;
        if element.gpu_instance_id > self.instance_data_idxs.len() as u32 {
            self.instance_data_idxs
                .resize(element.gpu_instance_id as usize + 1, u32::MAX);
        }
        assert!(self.instance_data_idxs[element.gpu_instance_id as usize] == u32::MAX);
        self.instance_data_idxs[element.gpu_instance_id as usize] = new_index;

        let mut instance_data = GpuInstanceData::default();
        instance_data.set_gpu_instance_id(element.gpu_instance_id.into());

        for layer in &mut self.default_layers {
            instance_data.set_state_id(0.into());
            layer.register_element(0, element);
        }
        self.gpu_instance_data.push(instance_data);
    }

    fn unregister_element(&mut self, gpu_instance_id: u32) {
        let removed_index = self.instance_data_idxs[gpu_instance_id as usize] as usize;
        self.instance_data_idxs[gpu_instance_id as usize] = u32::MAX;

        let removed_instance = self.gpu_instance_data.swap_remove(removed_index as usize);
        let removed_instance_id: u32 = removed_instance.gpu_instance_id().into();
        assert!(gpu_instance_id == removed_instance_id);

        if removed_index < self.gpu_instance_data.len() {
            let moved_instance_id: u32 = self.gpu_instance_data[removed_index as usize]
                .gpu_instance_id()
                .into();
            self.instance_data_idxs[moved_instance_id as usize] = removed_index as u32;
        }

        for layer in &mut self.default_layers {
            layer.unregister_element(removed_instance.state_id().into(), gpu_instance_id);
        }
    }

    fn prepare(&mut self, renderer: &Renderer) {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        let mut count_buffer_size: u64 = 0;
        let mut indirect_arg_buffer_size: u64 = 0;
        self.depth_count_buffer_count = 0;

        for (index, layer) in self.default_layers.iter_mut().enumerate() {
            layer.aggregate_offsets(
                &mut updater,
                &mut count_buffer_size,
                &mut indirect_arg_buffer_size,
            );
            if index == DefaultLayers::Depth as usize {
                self.depth_count_buffer_count = count_buffer_size;
            }
        }

        renderer.add_update_job_block(updater.job_blocks());

        let readback = self
            .culling_buffers
            .stats_buffer
            .begin_readback(renderer.device_context());

        readback.read_gpu_data(
            0,
            usize::MAX,
            u64::MAX,
            |data: &[CullingEfficiancyStats]| {
                self.culling_stats = data[0];
            },
        );
        self.culling_buffers.stats_buufer_readback = Some(readback);

        if count_buffer_size != 0 {
            create_or_replace_buffer(
                renderer.device_context(),
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
                renderer.device_context(),
                &mut self.culling_buffers.draw_args,
                std::mem::size_of::<u32>() as u64,
                indirect_arg_buffer_size * 5,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );
        }

        create_or_replace_buffer(
            renderer.device_context(),
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
            renderer.device_context(),
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
            renderer.device_context(),
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
            renderer.device_context(),
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
                renderer.device_context(),
                &mut self.culling_buffers.culled_instances,
                std::mem::size_of::<GpuInstanceData>() as u64,
                self.gpu_instance_data.len() as u64,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );

            create_or_replace_buffer(
                renderer.device_context(),
                &mut self.culling_buffers.culling_debug,
                std::mem::size_of::<CullingDebugData>() as u64,
                self.gpu_instance_data.len() as u64,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );

            create_or_replace_buffer(
                renderer.device_context(),
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

    fn cull(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        culling_buffers: &CullingArgBuffers,
        culling_options: &(IndirectDispatch, GatherPerfStats),
        culling_args: (u32, u32, Vec2),
        input_buffers: (&BufferView, &BufferView, &BufferView),
    ) {
        let indirect_dispatch = culling_options.0 .0;
        let gather_perf_stats = culling_options.1 .0;

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
            .pipeline_manager()
            .get_pipeline(pipeline_handle)
            .unwrap();
        cmd_buffer.bind_pipeline(pipeline);

        cmd_buffer.bind_descriptor_set(
            render_context.frame_descriptor_set().0,
            render_context.frame_descriptor_set().1,
        );

        cmd_buffer.bind_descriptor_set(
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

        let culling_descriptor_set_handle = render_context.write_descriptor_set(
            cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
            culling_descriptor_set.descriptor_refs(),
        );

        cmd_buffer.bind_descriptor_set(
            cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
            culling_descriptor_set_handle,
        );

        cmd_buffer.resource_barrier(
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
                self.depth_count_buffer_count * std::mem::size_of::<u32>() as u64;
            cmd_buffer.fill_buffer(&draw_count.buffer, 0, depth_count_size, 0);
        } else {
            cmd_buffer.fill_buffer(&draw_count.buffer, 0, !0, 0);
            cmd_buffer.fill_buffer(&culled_count.buffer, 0, 4, 0);
            cmd_buffer.fill_buffer(&culled_args.buffer, 0, 4, 0);
        }
        cmd_buffer.resource_barrier(
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
        culling_constant_data.set_hzb_pixel_extents(culling_args.2.into());
        culling_constant_data.set_options(options);

        cmd_buffer.push_constant(&culling_constant_data);

        if indirect_dispatch {
            cmd_buffer.dispatch_indirect(&dispatch_args.buffer, 0);
        } else {
            cmd_buffer.dispatch((self.gpu_instance_data.len() as u32 + 255) / 256, 1, 1);
        }

        cmd_buffer.resource_barrier(
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
    }

    pub(crate) fn gen_occlusion_and_cull(
        &self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        instance_manager: &GpuInstanceManager,
    ) {
        if self.culling_buffers.draw_count.is_none() {
            return;
        }

        let mut render_pass_data: Vec<RenderPassData> = vec![];
        for layer in &self.default_layers {
            let offset_base_va = layer.offsets_va();

            let mut pass_data = RenderPassData::default();
            pass_data.set_offset_base_va(offset_base_va.into());
            render_pass_data.push(pass_data);
        }

        let mut cmd_buffer = render_context.alloc_command_buffer();

        cmd_buffer.bind_index_buffer(
            &render_context
                .renderer()
                .static_buffer()
                .index_buffer_binding(),
        );
        cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

        let hzb_pixel_extents = render_surface.get_hzb_surface().hzb_pixel_extents();

        render_surface.init_hzb_if_needed(render_context, &mut cmd_buffer);

        let gpu_count_allocation = render_context.transient_buffer_allocator().copy_data(
            &(self.gpu_instance_data.len() as u32),
            ResourceUsage::AS_SHADER_RESOURCE,
        );
        let gpu_count_view =
            gpu_count_allocation.structured_buffer_view(std::mem::size_of::<u32>() as u64, true);

        let gpu_instance_allocation = render_context
            .transient_buffer_allocator()
            .copy_data_slice(&self.gpu_instance_data, ResourceUsage::AS_SHADER_RESOURCE);
        let gpu_instance_view = gpu_instance_allocation
            .structured_buffer_view(std::mem::size_of::<GpuInstanceData>() as u64, true);

        let render_pass_allocation = render_context
            .transient_buffer_allocator()
            .copy_data_slice(&render_pass_data, ResourceUsage::AS_SHADER_RESOURCE);
        let render_pass_view = render_pass_allocation
            .structured_buffer_view(std::mem::size_of::<RenderPassData>() as u64, true);

        self.culling_buffers.stats_buffer.clear_buffer(&cmd_buffer);

        // Cull using previous frame Hzb
        self.cull(
            render_context,
            &mut cmd_buffer,
            &self.culling_buffers,
            &(IndirectDispatch(false), GatherPerfStats(true)),
            (0, render_pass_data.len() as u32, hzb_pixel_extents),
            (&gpu_count_view, &gpu_instance_view, &render_pass_view),
        );

        cmd_buffer.begin_render_pass(
            &[],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_stencil_rt_view(),
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

        // Render initial depth buffer from last frame culling results
        self.draw(
            render_context,
            &mut cmd_buffer,
            DefaultLayers::Depth as usize,
        );

        cmd_buffer.end_render_pass();

        // Initial Hzb for current frame
        render_surface.generate_hzb(render_context, &mut cmd_buffer);

        // Rebind global vertex buffer after gen Hzb changes it
        cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

        // Retest elements culled from first pass against new Hzb
        self.cull(
            render_context,
            &mut cmd_buffer,
            &self.culling_buffers,
            &(IndirectDispatch(true), GatherPerfStats(true)),
            (0, render_pass_data.len() as u32, hzb_pixel_extents),
            (&gpu_count_view, &gpu_instance_view, &render_pass_view),
        );

        // Redraw depth istances that passed second cull pass
        cmd_buffer.begin_render_pass(
            &[],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_stencil_rt_view(),
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

        // Render initial depth buffer from last frame culling results
        self.draw(
            render_context,
            &mut cmd_buffer,
            DefaultLayers::Depth as usize,
        );

        cmd_buffer.end_render_pass();

        // Update Hzb from complete depth buffer
        render_surface.generate_hzb(render_context, &mut cmd_buffer);

        if let Some(readback) = &self.culling_buffers.stats_buufer_readback {
            self.culling_buffers
                .stats_buffer
                .copy_buffer_to_readback(&cmd_buffer, readback);
        }

        render_context
            .graphics_queue()
            .submit(&mut [cmd_buffer.finalize()], &[], &[], None);
    }

    pub(crate) fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        layer_id: usize,
    ) {
        self.default_layers[layer_id].draw(
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
    }

    pub(crate) fn render_end(&mut self) {
        let readback = std::mem::take(&mut self.culling_buffers.stats_buufer_readback);

        if let Some(readback) = readback {
            self.culling_buffers.stats_buffer.end_readback(readback);
        }
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
        let buffer_def = BufferDef {
            size: required_size,
            usage_flags,
            creation_flags: ResourceCreation::empty(),
        };
        let new_buffer = device_context.create_buffer(&buffer_def);

        let alloc_def = MemoryAllocationDef {
            memory_usage,
            always_mapped: false,
        };

        let allocation = MemoryAllocation::from_buffer(device_context, &new_buffer, &alloc_def);

        let srv_view_def =
            BufferViewDef::as_structured_buffer_with_offset(required_size, element_size, true, 0);
        let srv_view = BufferView::from_buffer(&new_buffer, &srv_view_def);

        let uav_view_def =
            BufferViewDef::as_structured_buffer_with_offset(required_size, element_size, false, 0);
        let uav_view = BufferView::from_buffer(&new_buffer, &uav_view_def);

        *buffer = Some(CullingArgBuffer {
            buffer: new_buffer,
            srv_view,
            uav_view,
            _allocation: allocation,
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

    let resterizer_state = RasterizerState {
        cull_mode: CullMode::Front,
        ..RasterizerState::default()
    };

    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(
            cgen::shader::depth_shader::ID,
            cgen::shader::depth_shader::NONE,
        ),
        move |device_context, shader| {
            device_context
                .create_graphics_pipeline(&GraphicsPipelineDef {
                    shader,
                    root_signature,
                    vertex_layout: &vertex_layout,
                    blend_state: &BlendState::default_alpha_disabled(),
                    depth_state: &depth_state,
                    rasterizer_state: &resterizer_state,
                    color_formats: &[],
                    sample_count: SampleCount::SampleCount1,
                    depth_stencil_format: Some(Format::D32_SFLOAT),
                    primitive_topology: PrimitiveTopology::TriangleList,
                })
                .unwrap()
        },
    )
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
        depth_write_enable: false,
        depth_compare_op: CompareOp::Equal,
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

    let resterizer_state = RasterizerState {
        cull_mode: CullMode::Front,
        ..RasterizerState::default()
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
                    blend_state: &BlendState::default_alpha_disabled(),
                    depth_state: &depth_state,
                    rasterizer_state: &resterizer_state,
                    color_formats: &[Format::R16G16B16A16_SFLOAT],
                    sample_count: SampleCount::SampleCount1,
                    depth_stencil_format: Some(Format::D32_SFLOAT),
                    primitive_topology: PrimitiveTopology::TriangleList,
                })
                .unwrap()
        },
    )
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
    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(shader::picking_shader::ID, shader::picking_shader::NONE),
        move |device_context, shader| {
            device_context
                .create_graphics_pipeline(&GraphicsPipelineDef {
                    shader,
                    root_signature,
                    vertex_layout: &vertex_layout,
                    blend_state: &BlendState::default_alpha_disabled(),
                    depth_state: &depth_state,
                    rasterizer_state: &RasterizerState::default(),
                    color_formats: &[Format::R16G16B16A16_SFLOAT],
                    sample_count: SampleCount::SampleCount1,
                    depth_stencil_format: None,
                    primitive_topology: PrimitiveTopology::TriangleList,
                })
                .unwrap()
        },
    )
}

fn build_culling_psos(pipeline_manager: &PipelineManager) -> (PipelineHandle, PipelineHandle) {
    let root_signature = cgen::pipeline_layout::CullingPipelineLayout::root_signature();

    (
        pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                shader::culling_shader::ID,
                shader::culling_shader::FIRST_PASS,
            ),
            move |device_context, shader| {
                device_context
                    .create_compute_pipeline(&ComputePipelineDef {
                        shader,
                        root_signature,
                    })
                    .unwrap()
            },
        ),
        pipeline_manager.register_pipeline(
            cgen::CRATE_ID,
            CGenShaderKey::make(
                shader::culling_shader::ID,
                shader::culling_shader::SECOND_PASS,
            ),
            move |device_context, shader| {
                device_context
                    .create_compute_pipeline(&ComputePipelineDef {
                        shader,
                        root_signature,
                    })
                    .unwrap()
            },
        ),
    )
}
