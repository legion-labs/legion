use std::slice;

use lgn_graphics_api::{
    BarrierQueueTransition, BlendState, Buffer, BufferBarrier, BufferCopy, BufferDef, BufferView,
    BufferViewDef, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState, DeviceContext,
    Format, GraphicsPipelineDef, LoadOp, MemoryAllocation, MemoryAllocationDef, MemoryUsage,
    Pipeline, PrimitiveTopology, RasterizerState, ResourceCreation, ResourceState, ResourceUsage,
    SampleCount, StencilOp, StoreOp, VertexLayout,
};
use lgn_transform::components::Transform;

use crate::{
    cgen::{self, cgen_type::PickingData},
    components::{
        CameraComponent, LightComponent, ManipulatorComponent, PickedComponent, RenderSurface,
        StaticMesh,
    },
    hl_gfx_api::HLCommandBuffer,
    picking::{ManipulatorManager, PickingManager, PickingState},
    resources::{GpuSafePool, OnFrameEventHandler},
    RenderContext, RenderHandle, Renderer,
};

use lgn_math::Mat4;

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
    pipeline: Pipeline,

    readback_buffer_pools: GpuSafePool<ReadbackBufferPool>,

    count_buffer: Buffer,
    _count_allocation: MemoryAllocation,
    count_rw_view: BufferView,

    picked_buffer: Buffer,
    _picked_allocation: MemoryAllocation,
    picked_rw_view: BufferView,
}

impl PickingRenderPass {
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        let root_signature = cgen::pipeline_layout::PickingPipelineLayout::root_signature();
        let shader = renderer.prepare_vs_ps(String::from("crate://renderer/shaders/picking.hlsl"));

        //
        // Pipeline state
        //
        let vertex_layout = VertexLayout {
            attributes: vec![],
            buffers: vec![],
        };

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
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

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
            pipeline,
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
    pub fn render(
        &mut self,
        picking_manager: &PickingManager,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, Option<&PickedComponent>)],
        manipulator_meshes: &[(&StaticMesh, &Transform, &ManipulatorComponent)],
        lights: &[(&LightComponent, &Transform)],
        light_picking_mesh: &StaticMesh,
        camera: &CameraComponent,
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

            cmd_buffer.bind_pipeline(&self.pipeline);
            cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
            cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

            let mut picking_descriptor_set = cgen::descriptor_set::PickingDescriptorSet::default();
            picking_descriptor_set.set_picked_count(&self.count_rw_view);
            picking_descriptor_set.set_picked_objects(&self.picked_rw_view);
            let picking_descriptor_set_handle =
                render_context.write_descriptor_set(&picking_descriptor_set);
            cmd_buffer.bind_descriptor_set_handle(picking_descriptor_set_handle);

            let (view_matrix, projection_matrix) = camera.build_view_projection(
                render_surface.extents().width() as f32,
                render_surface.extents().height() as f32,
            );

            for (_index, (static_mesh, transform, manipulator)) in
                manipulator_meshes.iter().enumerate()
            {
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
                        None,
                        picking_distance,
                        static_mesh,
                        &cmd_buffer,
                    );
                }
            }

            for (_index, (static_mesh, _picked)) in static_meshes.iter().enumerate() {
                let picking_distance = 1.0;
                let custom_world = Mat4::IDENTITY;

                render_mesh(
                    &custom_world,
                    None,
                    picking_distance,
                    static_mesh,
                    &cmd_buffer,
                );
            }

            for (light, transform) in lights {
                let picking_distance = 1.0;
                let custom_world = transform.with_scale(transform.scale * 0.2).compute_matrix();
                render_mesh(
                    &custom_world,
                    Some(light.picking_id),
                    picking_distance,
                    light_picking_mesh,
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
        readback: &RenderHandle<ReadbackBufferPool>,
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
    picking_id: Option<u32>,
    picking_distance: f32,
    static_mesh: &StaticMesh,
    cmd_buffer: &HLCommandBuffer<'_>,
) {
    let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();
    push_constant_data.set_world((*custom_world).into());
    push_constant_data.set_picking_distance(picking_distance.into());
    push_constant_data.set_vertex_offset(static_mesh.vertex_offset.into());
    push_constant_data.set_world_offset(static_mesh.world_offset.into());
    push_constant_data.set_picking_id(
        if let Some(id) = picking_id {
            id
        } else {
            static_mesh.picking_id
        }
        .into(),
    );
    push_constant_data.set_picking_distance(picking_distance.into());

    cmd_buffer.push_constant(&push_constant_data);

    cmd_buffer.draw(static_mesh.num_vertices, 0);
}
