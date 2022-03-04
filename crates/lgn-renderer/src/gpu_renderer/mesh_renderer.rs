use lgn_app::{App, CoreStage, EventReader, Plugin};
use lgn_ecs::prelude::{Query, Res, ResMut};
use lgn_graphics_api::{
    BarrierQueueTransition, BlendState, Buffer, BufferBarrier, BufferDef, BufferView,
    BufferViewDef, CompareOp, ComputePipelineDef, DepthState, DeviceContext, Format,
    GraphicsPipelineDef, MemoryAllocation, MemoryAllocationDef, MemoryUsage, PrimitiveTopology,
    RasterizerState, ResourceCreation, ResourceState, ResourceUsage, SampleCount, StencilOp,
    VertexAttributeRate, VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;

use crate::{
    cgen::{
        self,
        cgen_type::{GpuInstanceData, RenderPassData},
        shader,
    },
    components::{CameraComponent, RenderSurface},
    hl_gfx_api::HLCommandBuffer,
    labels::RenderStage,
    resources::{
        PipelineHandle, PipelineManager, UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
    },
    RenderContext, Renderer,
};

use super::{GpuInstanceEvent, RenderElement, RenderLayer, RenderStateSet};

#[derive(Default)]
pub struct MeshRendererPlugin {}

pub(crate) enum DefaultLayers {
    Opaque = 0,
    Picking,
}

impl Plugin for MeshRendererPlugin {
    fn build(&self, app: &mut App) {
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
fn prepare(
    renderer: Res<'_, Renderer>,
    mut mesh_renderer: ResMut<'_, MeshRenderer>,
    surfaces: Query<'_, '_, &mut RenderSurface>,
    cameras: Query<'_, '_, &CameraComponent>,
) {
    let cameras = cameras.iter().collect::<Vec<&CameraComponent>>();
    let surfaces = surfaces.iter().collect::<Vec<&RenderSurface>>();

    if !cameras.is_empty() && !surfaces.is_empty() {
        let aspect_ratio =
            surfaces[0].extents().width() as f32 / surfaces[0].extents().height() as f32;
        mesh_renderer.prepare(&renderer, cameras[0], aspect_ratio);
    };
}

struct CullingArgBuffer {
    buffer: Buffer,
    buffer_view: BufferView,
    _allocation: MemoryAllocation,
}

pub struct MeshRenderer {
    default_layers: Vec<RenderLayer>,

    indirect_arg_buffer: Option<CullingArgBuffer>,
    count_buffer: Option<CullingArgBuffer>,

    instance_data_idxs: Vec<u32>,
    gpu_instance_data: Vec<GpuInstanceData>,
    render_pass_data: Vec<RenderPassData>,

    culling_shader: Option<PipelineHandle>,

    tmp_batch_ids: Vec<u32>,
    tmp_pipeline_handles: Vec<PipelineHandle>,
}

impl MeshRenderer {
    pub(crate) fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            default_layers: vec![
                RenderLayer::new(allocator, false),
                RenderLayer::new(allocator, false),
            ],
            indirect_arg_buffer: None,
            count_buffer: None,
            instance_data_idxs: vec![],
            gpu_instance_data: vec![],
            render_pass_data: vec![],
            culling_shader: None,
            tmp_batch_ids: vec![],
            tmp_pipeline_handles: vec![],
        }
    }

    fn initialize_psos(&mut self, pipeline_manager: &PipelineManager) {
        if self.culling_shader.is_none() {
            self.culling_shader = Some(build_culling_pso(pipeline_manager));

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

    fn prepare(&mut self, renderer: &Renderer, camera: &CameraComponent, aspect_ratio: f32) {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        let mut count_buffer_size: u64 = 0;
        let mut indirect_arg_buffer_size: u64 = 0;

        self.render_pass_data.clear();
        for layer in &mut self.default_layers {
            let offset_base_va = layer.aggregate_offsets(
                &mut updater,
                &mut count_buffer_size,
                &mut indirect_arg_buffer_size,
            );

            let mut pass_data = RenderPassData::default();
            pass_data.set_culling_planes(camera.build_culling_planes(aspect_ratio));
            pass_data.set_offset_base_va(offset_base_va.into());
            self.render_pass_data.push(pass_data);
        }

        renderer.add_update_job_block(updater.job_blocks());

        if count_buffer_size != 0 {
            create_or_replace_buffer(
                renderer.device_context(),
                &mut self.count_buffer,
                count_buffer_size,
                false,
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
                &mut self.indirect_arg_buffer,
                indirect_arg_buffer_size * 5,
                false,
                ResourceUsage::AS_INDIRECT_BUFFER
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS,
                MemoryUsage::GpuOnly,
            );
        }
    }

    pub(crate) fn cull(&self, render_context: &RenderContext<'_>) {
        if self.count_buffer.is_none() || self.indirect_arg_buffer.is_none() {
            return;
        }

        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.culling_shader.unwrap())
            .unwrap();

        let mut cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.bind_pipeline(pipeline);

        cmd_buffer.bind_descriptor_set(
            render_context.frame_descriptor_set().0,
            render_context.frame_descriptor_set().1,
        );

        let count_buffer = self.count_buffer.as_ref().unwrap();
        let indirect_arg_buffer = self.indirect_arg_buffer.as_ref().unwrap();

        let mut culling_descriptor_set = cgen::descriptor_set::CullingDescriptorSet::default();
        culling_descriptor_set.set_count_buffer(&count_buffer.buffer_view);
        culling_descriptor_set.set_indirect_arg_buffer(&indirect_arg_buffer.buffer_view);

        let gpu_instance_allocation = render_context
            .transient_buffer_allocator()
            .copy_data_slice(&self.gpu_instance_data, ResourceUsage::AS_SHADER_RESOURCE);
        let gpu_instance_view = gpu_instance_allocation
            .structured_buffer_view(std::mem::size_of::<GpuInstanceData>() as u64, true);
        culling_descriptor_set.set_gpu_instance_data(&gpu_instance_view);

        let render_pass_allocation = render_context
            .transient_buffer_allocator()
            .copy_data_slice(&self.render_pass_data, ResourceUsage::AS_SHADER_RESOURCE);
        let render_pass_view = render_pass_allocation
            .structured_buffer_view(std::mem::size_of::<RenderPassData>() as u64, true);
        culling_descriptor_set.set_render_pass_data(&render_pass_view);

        let culling_descriptor_set_handle = render_context.write_descriptor_set(
            cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
            culling_descriptor_set.descriptor_refs(),
        );

        cmd_buffer.bind_descriptor_set(
            cgen::descriptor_set::CullingDescriptorSet::descriptor_set_layout(),
            culling_descriptor_set_handle,
        );

        let mut culling_constant_data = cgen::cgen_type::CullingPushConstantData::default();
        culling_constant_data.set_num_gpu_instances((self.gpu_instance_data.len() as u32).into());
        culling_constant_data.set_num_render_passes((self.render_pass_data.len() as u32).into());

        cmd_buffer.push_constant(&culling_constant_data);

        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &count_buffer.buffer,
                src_state: ResourceState::INDIRECT_ARGUMENT,
                dst_state: ResourceState::COPY_DST,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );

        cmd_buffer.fill_buffer(&count_buffer.buffer, 0, !0, 0);

        cmd_buffer.resource_barrier(
            &[
                BufferBarrier {
                    buffer: &count_buffer.buffer,
                    src_state: ResourceState::COPY_DST,
                    dst_state: ResourceState::UNORDERED_ACCESS,
                    queue_transition: BarrierQueueTransition::None,
                },
                BufferBarrier {
                    buffer: &indirect_arg_buffer.buffer,
                    src_state: ResourceState::INDIRECT_ARGUMENT,
                    dst_state: ResourceState::UNORDERED_ACCESS,
                    queue_transition: BarrierQueueTransition::None,
                },
            ],
            &[],
        );

        cmd_buffer.dispatch((self.gpu_instance_data.len() as u32 + 255) / 256, 1, 1);

        cmd_buffer.resource_barrier(
            &[
                BufferBarrier {
                    buffer: &count_buffer.buffer,
                    src_state: ResourceState::UNORDERED_ACCESS,
                    dst_state: ResourceState::INDIRECT_ARGUMENT,
                    queue_transition: BarrierQueueTransition::None,
                },
                BufferBarrier {
                    buffer: &indirect_arg_buffer.buffer,
                    src_state: ResourceState::UNORDERED_ACCESS,
                    dst_state: ResourceState::INDIRECT_ARGUMENT,
                    queue_transition: BarrierQueueTransition::None,
                },
            ],
            &[],
        );

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
            self.indirect_arg_buffer
                .as_ref()
                .map(|buffer| &buffer.buffer),
            self.count_buffer.as_ref().map(|buffer| &buffer.buffer),
        );
    }
}

fn create_or_replace_buffer(
    device_context: &DeviceContext,
    buffer: &mut Option<CullingArgBuffer>,
    element_count: u64,
    read_only: bool,
    usage_flags: ResourceUsage,
    memory_usage: MemoryUsage,
) {
    let struct_size = std::mem::size_of::<u32>() as u64;
    let required_size = element_count * struct_size;

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

        let buffer_view_def = BufferViewDef::as_structured_buffer_with_offset(
            required_size,
            struct_size,
            read_only,
            0,
        );
        let buffer_view = BufferView::from_buffer(&new_buffer, &buffer_view_def);

        *buffer = Some(CullingArgBuffer {
            buffer: new_buffer,
            buffer_view,
            _allocation: allocation,
        });
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
                    blend_state: &BlendState::default_alpha_enabled(),
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

fn build_culling_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
    let root_signature = cgen::pipeline_layout::CullingPipelineLayout::root_signature();

    pipeline_manager.register_pipeline(
        cgen::CRATE_ID,
        CGenShaderKey::make(shader::culling_shader::ID, shader::culling_shader::NONE),
        move |device_context, shader| {
            device_context
                .create_compute_pipeline(&ComputePipelineDef {
                    shader,
                    root_signature,
                })
                .unwrap()
        },
    )
}
