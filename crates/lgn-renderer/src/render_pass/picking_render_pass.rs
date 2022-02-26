use std::slice;

use lgn_core::Handle;

use lgn_graphics_api::{
    BarrierQueueTransition, Buffer, BufferBarrier, BufferCopy, BufferDef, BufferView,
    BufferViewDef, ColorClearValue, ColorRenderTargetBinding, DeviceContext, LoadOp,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, ResourceCreation, ResourceState,
    ResourceUsage, StoreOp,
};
use lgn_math::Mat4;
use lgn_transform::components::GlobalTransform;

use crate::{
    cgen::{self, cgen_type::PickingData},
    components::{
        CameraComponent, LightComponent, ManipulatorComponent, RenderSurface, VisualComponent,
    },
    gpu_renderer::{DefaultLayers, GpuInstanceManager, MeshRenderer},
    hl_gfx_api::HLCommandBuffer,
    picking::{ManipulatorManager, PickingManager, PickingState},
    resources::{DefaultMeshType, GpuSafePool, MeshManager, ModelManager, OnFrameEventHandler},
    RenderContext,
};

struct ReadbackBufferPool {
    device_context: DeviceContext,
    picking_manager: PickingManager,

    count_buffer: Buffer,
    count_allocation: MemoryAllocation,
    picked_buffer: Buffer,
    picked_allocation: MemoryAllocation,

    cpu_frame_for_results: u64,
}

impl ReadbackBufferPool {
    pub(crate) fn new(device_context: &DeviceContext, picking_manager: &PickingManager) -> Self {
        let count_buffer_def = BufferDef {
            size: 4,
            usage_flags: ResourceUsage::AS_TRANSFERABLE,
            creation_flags: ResourceCreation::empty(),
        };

        let count_buffer = device_context.create_buffer(&count_buffer_def);

        let count_alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::GpuToCpu,
            always_mapped: false,
        };

        let count_allocation =
            MemoryAllocation::from_buffer(device_context, &count_buffer, &count_alloc_def);

        let picked_buffer_def = BufferDef {
            size: 16 * 1024,
            usage_flags: ResourceUsage::AS_TRANSFERABLE,
            creation_flags: ResourceCreation::empty(),
        };

        let picked_buffer = device_context.create_buffer(&picked_buffer_def);

        let picked_alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::GpuToCpu,
            always_mapped: false,
        };

        let picked_allocation =
            MemoryAllocation::from_buffer(device_context, &picked_buffer, &picked_alloc_def);

        Self {
            device_context: device_context.clone(),
            picking_manager: picking_manager.clone(),
            count_buffer,
            count_allocation,
            picked_buffer,
            picked_allocation,
            cpu_frame_for_results: u64::MAX,
        }
    }

    fn get_gpu_results(&mut self, picked_cpu_frame_no: u64) {
        if self.cpu_frame_for_results != u64::MAX
            && self.cpu_frame_for_results == picked_cpu_frame_no
        {
            let mapping_info = self.count_allocation.map_buffer(&self.device_context);

            let count;
            #[allow(unsafe_code)]
            unsafe {
                count = u32::from(*mapping_info.data_ptr());
            }

            let mapping_info = self.picked_allocation.map_buffer(&self.device_context);

            #[allow(unsafe_code)]
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                self.picking_manager.set_picked(slice::from_raw_parts(
                    mapping_info.data_ptr() as *const PickingData,
                    count as usize,
                ));
            }
        }
        self.cpu_frame_for_results = u64::MAX;
    }

    fn sent_to_gpu(&mut self, cpu_frame_for_results: u64) {
        self.cpu_frame_for_results = cpu_frame_for_results;
    }
}

impl OnFrameEventHandler for ReadbackBufferPool {
    fn on_begin_frame(&mut self) {}

    fn on_end_frame(&mut self) {}
}

pub struct PickingRenderPass {
    readback_buffer_pools: GpuSafePool<ReadbackBufferPool>,

    count_buffer: Buffer,
    _count_allocation: MemoryAllocation,
    count_rw_view: BufferView,

    picked_buffer: Buffer,
    _picked_allocation: MemoryAllocation,
    picked_rw_view: BufferView,
}

impl PickingRenderPass {
    pub fn new(device_context: &DeviceContext) -> Self {
        let count_buffer_def = BufferDef {
            size: 4,

            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS
                | ResourceUsage::AS_TRANSFERABLE,
            creation_flags: ResourceCreation::empty(),
        };

        let count_buffer = device_context.create_buffer(&count_buffer_def);

        let count_alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::GpuOnly,
            always_mapped: false,
        };

        let count_allocation =
            MemoryAllocation::from_buffer(device_context, &count_buffer, &count_alloc_def);

        let count_rw_view_def =
            BufferViewDef::as_structured_buffer(count_buffer.definition(), 4, false);
        let count_rw_view = BufferView::from_buffer(&count_buffer, &count_rw_view_def);

        let picked_buffer_def = BufferDef {
            size: 16 * 1024,

            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_UNORDERED_ACCESS,
            creation_flags: ResourceCreation::empty(),
        };

        let picked_buffer = device_context.create_buffer(&picked_buffer_def);

        let picked_alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::GpuOnly,
            always_mapped: false,
        };

        let picked_allocation =
            MemoryAllocation::from_buffer(device_context, &picked_buffer, &picked_alloc_def);

        let picked_rw_view_def =
            BufferViewDef::as_structured_buffer(picked_buffer.definition(), 16, false);
        let picked_rw_view = BufferView::from_buffer(&picked_buffer, &picked_rw_view_def);

        Self {
            readback_buffer_pools: GpuSafePool::new(3),
            count_buffer,
            _count_allocation: count_allocation,
            count_rw_view,
            picked_buffer,
            _picked_allocation: picked_allocation,
            picked_rw_view,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        picking_manager: &PickingManager,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        instance_manager: &GpuInstanceManager,
        manipulator_meshes: &[(&VisualComponent, &GlobalTransform, &ManipulatorComponent)],
        lights: &[(&LightComponent, &GlobalTransform)],
        mesh_manager: &MeshManager,
        model_manager: &ModelManager,
        camera: &CameraComponent,
        mesh_renerer: &MeshRenderer,
    ) {
        self.readback_buffer_pools.begin_frame();
        let mut readback = self.readback_buffer_pools.acquire_or_create(|| {
            ReadbackBufferPool::new(render_context.renderer().device_context(), picking_manager)
        });

        readback.get_gpu_results(picking_manager.frame_no_picked());

        if picking_manager.picking_state() == PickingState::Rendering {
            let mut cmd_buffer = render_context.alloc_command_buffer();

            render_surface.transition_to(&cmd_buffer, ResourceState::RENDER_TARGET);

            self.init_picking_results(&cmd_buffer);

            cmd_buffer.begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.render_target_view(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue::default(),
                }],
                &None,
            );

            let pipeline = render_context
                .pipeline_manager()
                .get_pipeline(mesh_renerer.get_tmp_pso_handle(DefaultLayers::Picking as usize))
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

            cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

            let mut picking_descriptor_set = cgen::descriptor_set::PickingDescriptorSet::default();
            picking_descriptor_set.set_picked_count(&self.count_rw_view);
            picking_descriptor_set.set_picked_objects(&self.picked_rw_view);
            let picking_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                picking_descriptor_set.descriptor_refs(),
            );
            cmd_buffer.bind_descriptor_set(
                cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                picking_descriptor_set_handle,
            );

            let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();
            push_constant_data.set_picking_distance(1.0.into());
            push_constant_data.set_use_gpu_pipeline(1.into());

            cmd_buffer.push_constant(&push_constant_data);

            mesh_renerer.draw(
                render_context,
                &mut cmd_buffer,
                DefaultLayers::Picking as usize,
            );

            let (view_matrix, projection_matrix) = camera.build_view_projection(
                render_surface.extents().width() as f32,
                render_surface.extents().height() as f32,
            );

            for (_index, (visual, transform, manipulator)) in manipulator_meshes.iter().enumerate()
            {
                let (model_meta_data, _ready) = model_manager.get_model_meta_data(visual);
                for mesh in &model_meta_data.meshes {
                    if manipulator.active {
                        let picking_distance = 50.0;
                        let custom_world = ManipulatorManager::scale_manipulator_for_viewport(
                            transform,
                            &manipulator.local_transform,
                            &view_matrix,
                            &projection_matrix,
                        );

                        render_mesh(
                            &custom_world,
                            manipulator.picking_id,
                            picking_distance,
                            mesh.mesh_id as u32,
                            mesh_manager,
                            &cmd_buffer,
                        );
                    }
                }
            }

            for (light, transform) in lights {
                let picking_distance = 1.0;
                let custom_world = transform.with_scale(transform.scale * 0.2).compute_matrix();
                render_mesh(
                    &custom_world,
                    light.picking_id,
                    picking_distance,
                    DefaultMeshType::Sphere as u32,
                    mesh_manager,
                    &cmd_buffer,
                );
            }

            cmd_buffer.end_render_pass();

            self.copy_picking_results_to_readback(&cmd_buffer, &readback);

            {
                let graphics_queue = render_context.graphics_queue();
                graphics_queue.submit(&mut [cmd_buffer.finalize()], &[], &[], None);
            }

            readback.sent_to_gpu(picking_manager.frame_no_for_picking());
        }

        self.readback_buffer_pools.release(readback);
        self.readback_buffer_pools.end_frame();
    }

    fn init_picking_results(&mut self, cmd_buffer: &HLCommandBuffer<'_>) {
        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &self.count_buffer,
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::COPY_DST,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );

        cmd_buffer.fill_buffer(&self.count_buffer, 0, 4, 0);

        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &self.count_buffer,
                src_state: ResourceState::COPY_DST,
                dst_state: ResourceState::UNORDERED_ACCESS,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );
    }

    fn copy_picking_results_to_readback(
        &mut self,
        cmd_buffer: &HLCommandBuffer<'_>,
        readback: &Handle<ReadbackBufferPool>,
    ) {
        cmd_buffer.resource_barrier(
            &[
                BufferBarrier {
                    buffer: &self.count_buffer,
                    src_state: ResourceState::UNORDERED_ACCESS,
                    dst_state: ResourceState::COPY_SRC,
                    queue_transition: BarrierQueueTransition::None,
                },
                BufferBarrier {
                    buffer: &self.picked_buffer,
                    src_state: ResourceState::UNORDERED_ACCESS,
                    dst_state: ResourceState::COPY_SRC,
                    queue_transition: BarrierQueueTransition::None,
                },
            ],
            &[],
        );

        cmd_buffer.copy_buffer_to_buffer(
            &self.count_buffer,
            &readback.count_buffer,
            &[BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: 4,
            }],
        );

        cmd_buffer.copy_buffer_to_buffer(
            &self.picked_buffer,
            &readback.picked_buffer,
            &[BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: 1024 * 16,
            }],
        );

        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &self.picked_buffer,
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::UNORDERED_ACCESS,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );
    }
}
fn render_mesh(
    custom_world: &Mat4,
    picking_id: u32,
    picking_distance: f32,
    mesh_id: u32,
    mesh_manager: &MeshManager,
    cmd_buffer: &HLCommandBuffer<'_>,
) {
    let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();
    let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh_id);
    push_constant_data.set_world((*custom_world).into());
    push_constant_data.set_mesh_description_offset(mesh_meta_data.mesh_description_offset.into());
    push_constant_data.set_picking_id(picking_id.into());
    push_constant_data.set_picking_distance(picking_distance.into());
    push_constant_data.set_use_gpu_pipeline(0.into());

    cmd_buffer.push_constant(&push_constant_data);

    cmd_buffer.draw(mesh_meta_data.draw_call_count, 0);
}
