use std::{num::NonZeroU32, slice};

use lgn_graphics_api::{
    BarrierQueueTransition, BlendState, Buffer, BufferBarrier, BufferCopy, BufferDef, BufferView,
    BufferViewDef, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState, DescriptorDef,
    DescriptorRef, DescriptorSetLayoutDef, DeviceContext, Format, GraphicsPipelineDef, LoadOp,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, Pipeline, PipelineType, PrimitiveTopology,
    PushConstantDef, QueueType, RasterizerState, ResourceCreation, ResourceState, ResourceUsage,
    RootSignature, RootSignatureDef, SampleCount, ShaderPackage, ShaderStageDef, ShaderStageFlags,
    StencilOp, StoreOp, VertexLayout, MAX_DESCRIPTOR_SET_LAYOUTS,
};

use lgn_math::{Mat4, Vec3};
use lgn_pso_compiler::{CompileParams, EntryPoint, ShaderSource};
use lgn_transform::prelude::Transform;

use crate::{
    components::{PickedComponent, RenderSurface, StaticMesh},
    resources::{CommandBufferHandle, GpuSafePool, OnFrameEventHandler},
    RenderContext, RenderHandle, Renderer,
};

use super::{PickingManager, PickingState};

#[derive(Clone, Copy)]
pub(super) struct PickingData {
    pub(super) picking_pos: Vec3,
    pub(super) picking_id: u32,
}

impl Default for PickingData {
    fn default() -> Self {
        Self {
            picking_pos: Vec3::ZERO,
            picking_id: 0,
        }
    }
}

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
            queue_type: QueueType::Graphics,
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
            queue_type: QueueType::Graphics,
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
    root_signature: RootSignature,
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
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        //
        // Shaders
        //
        let shader_compiler = renderer.shader_compiler();

        let shader_source =
            String::from_utf8(include_bytes!("../../shaders/picking.hlsl").to_vec()).unwrap();

        let shader_build_result = shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Code(shader_source),
                glob_defines: Vec::new(),
                entry_points: vec![
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_vs".to_owned(),
                        target_profile: "vs_6_0".to_owned(),
                    },
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_ps".to_owned(),
                        target_profile: "ps_6_0".to_owned(),
                    },
                ],
            })
            .unwrap();

        let vert_shader_module = device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let frag_shader_module = device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[1].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let shader = device_context
            .create_shader(
                vec![
                    ShaderStageDef {
                        entry_point: "main_vs".to_owned(),
                        shader_stage: ShaderStageFlags::VERTEX,
                        shader_module: vert_shader_module,
                    },
                    ShaderStageDef {
                        entry_point: "main_ps".to_owned(),
                        shader_stage: ShaderStageFlags::FRAGMENT,
                        shader_module: frag_shader_module,
                    },
                ],
                &shader_build_result.pipeline_reflection,
            )
            .unwrap();

        //
        // Root signature
        //
        let mut descriptor_set_layouts = Vec::new();
        for set_index in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
            let shader_resources: Vec<_> = shader_build_result
                .pipeline_reflection
                .shader_resources
                .iter()
                .filter(|x| x.set_index as usize == set_index)
                .collect();

            if !shader_resources.is_empty() {
                let descriptor_defs = shader_resources
                    .iter()
                    .map(|sr| DescriptorDef {
                        name: sr.name.clone(),
                        binding: sr.binding,
                        shader_resource_type: sr.shader_resource_type,
                        array_size: sr.element_count,
                    })
                    .collect();

                let def = DescriptorSetLayoutDef {
                    frequency: set_index as u32,
                    descriptor_defs,
                };
                let descriptor_set_layout =
                    device_context.create_descriptorset_layout(&def).unwrap();
                descriptor_set_layouts.push(descriptor_set_layout);
            }
        }

        let root_signature_def = RootSignatureDef {
            descriptor_set_layouts: descriptor_set_layouts.clone(),
            push_constant_def: shader_build_result
                .pipeline_reflection
                .push_constant
                .map(|x| PushConstantDef {
                    used_in_shader_stages: x.used_in_shader_stages,
                    size: NonZeroU32::new(x.size).unwrap(),
                }),
        };

        let root_signature = device_context
            .create_root_signature(&root_signature_def)
            .unwrap();

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
                root_signature: &root_signature,
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
            queue_type: QueueType::Graphics,
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
        let count_rw_view = BufferView::from_buffer(&count_buffer, &count_rw_view_def).unwrap();

        let picked_buffer_def = BufferDef {
            size: 16 * 1024,
            queue_type: QueueType::Graphics,
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
        let picked_rw_view = BufferView::from_buffer(&picked_buffer, &picked_rw_view_def).unwrap();

        Self {
            root_signature,
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

    pub fn render(
        &mut self,
        picking_manager: &PickingManager,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&StaticMesh, Option<&PickedComponent>)],
        camera_transform: &Transform,
    ) {
        self.readback_buffer_pools.begin_frame();
        let mut readback = self.readback_buffer_pools.acquire_or_create(|| {
            ReadbackBufferPool::new(render_context.renderer().device_context(), picking_manager)
        });

        readback.get_gpu_results(picking_manager.frame_no_picked());

        if picking_manager.picking_state() == PickingState::Rendering {
            let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);
            cmd_buffer.begin().unwrap();

            render_surface.transition_to(&cmd_buffer, ResourceState::RENDER_TARGET);

            self.init_picking_results(&cmd_buffer);

            cmd_buffer
                .cmd_begin_render_pass(
                    &[ColorRenderTargetBinding {
                        texture_view: render_surface.render_target_view(),
                        load_op: LoadOp::Clear,
                        store_op: StoreOp::Store,
                        clear_value: ColorClearValue::default(),
                    }],
                    &None,
                )
                .unwrap();

            cmd_buffer.cmd_bind_pipeline(&self.pipeline).unwrap();

            let descriptor_set_layout = &self
                .pipeline
                .root_signature()
                .definition()
                .descriptor_set_layouts[0];

            let fov_y_radians: f32 = 45.0;
            let width = render_surface.extents().width() as f32;
            let height = render_surface.extents().height() as f32;
            let aspect_ratio: f32 = width / height;
            let z_near: f32 = 0.01;
            let z_far: f32 = 100.0;
            let projection_matrix =
                Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

            let view_matrix = Mat4::look_at_lh(
                camera_transform.translation,
                camera_transform.translation + camera_transform.forward(),
                Vec3::new(0.0, 1.0, 0.0),
            );

            let view_proj_matrix = projection_matrix * view_matrix;
            let inv_view_proj_matrix = view_proj_matrix.inverse();

            let mut transient_allocator = render_context.acquire_transient_buffer_allocator();

            for (_index, (static_mesh_component, _picked_component)) in
                static_meshes.iter().enumerate()
            {
                let mut constant_data: [f32; 39] = [0.0; 39];
                view_proj_matrix.write_cols_to_slice(&mut constant_data[0..]);
                inv_view_proj_matrix.write_cols_to_slice(&mut constant_data[16..]);

                let screen_rect = picking_manager.screen_rect();
                constant_data[32] = screen_rect.x;
                constant_data[33] = screen_rect.y;
                constant_data[34] = 1.0 / screen_rect.x;
                constant_data[35] = 1.0 / screen_rect.y;

                let cursor_pos = picking_manager.current_cursor_pos();
                constant_data[36] = cursor_pos.x;
                constant_data[37] = cursor_pos.y;

                constant_data[38] = 1.0;

                let sub_allocation =
                    transient_allocator.copy_data(&constant_data, ResourceUsage::AS_CONST_BUFFER);

                let const_buffer_view = sub_allocation.const_buffer_view();

                let mut descriptor_set_writer =
                    render_context.alloc_descriptor_set(descriptor_set_layout);

                descriptor_set_writer
                    .set_descriptors_by_name(
                        "const_data",
                        &[DescriptorRef::BufferView(&const_buffer_view)],
                    )
                    .unwrap();

                let static_buffer_ro_view = render_context.renderer().static_buffer_ro_view();
                descriptor_set_writer
                    .set_descriptors_by_name(
                        "static_buffer",
                        &[DescriptorRef::BufferView(&static_buffer_ro_view)],
                    )
                    .unwrap();

                descriptor_set_writer
                    .set_descriptors_by_name(
                        "picked_count",
                        &[DescriptorRef::BufferView(&self.count_rw_view)],
                    )
                    .unwrap();

                descriptor_set_writer
                    .set_descriptors_by_name(
                        "picked_objects",
                        &[DescriptorRef::BufferView(&self.picked_rw_view)],
                    )
                    .unwrap();

                let descriptor_set_handle =
                    descriptor_set_writer.flush(render_context.renderer().device_context());

                cmd_buffer
                    .cmd_bind_descriptor_set_handle(
                        PipelineType::Graphics,
                        &self.root_signature,
                        descriptor_set_layout.definition().frequency,
                        descriptor_set_handle,
                    )
                    .unwrap();

                let mut push_constant_data: [u32; 3] = [0; 3];
                push_constant_data[0] = static_mesh_component.vertex_offset;
                push_constant_data[1] = static_mesh_component.world_offset;
                push_constant_data[2] = static_mesh_component.picking_id;

                cmd_buffer
                    .cmd_push_constants(&self.root_signature, &push_constant_data)
                    .unwrap();

                cmd_buffer
                    .cmd_draw(static_mesh_component.num_verticies, 0)
                    .unwrap();
            }

            render_context.release_transient_buffer_allocator(transient_allocator);

            cmd_buffer.cmd_end_render_pass().unwrap();

            self.copy_picking_results_to_readback(&cmd_buffer, &readback);

            cmd_buffer.end().unwrap();

            {
                let graphics_queue = render_context.renderer().queue(QueueType::Graphics);
                graphics_queue
                    .submit(&[&cmd_buffer], &[], &[], None)
                    .unwrap();
            }

            render_context.release_cmd_buffer(cmd_buffer);

            readback.sent_to_gpu(picking_manager.frame_no_for_picking());
        }

        self.readback_buffer_pools.release(readback);
        self.readback_buffer_pools.end_frame();
    }

    fn init_picking_results(&mut self, cmd_buffer: &CommandBufferHandle) {
        cmd_buffer
            .cmd_resource_barrier(
                &[BufferBarrier {
                    buffer: &self.count_buffer,
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    queue_transition: BarrierQueueTransition::None,
                }],
                &[],
            )
            .unwrap();

        cmd_buffer.cmd_fill_buffer(&self.count_buffer, 0, 4, 0);

        cmd_buffer
            .cmd_resource_barrier(
                &[BufferBarrier {
                    buffer: &self.count_buffer,
                    src_state: ResourceState::COPY_DST,
                    dst_state: ResourceState::UNORDERED_ACCESS,
                    queue_transition: BarrierQueueTransition::None,
                }],
                &[],
            )
            .unwrap();
    }

    fn copy_picking_results_to_readback(
        &mut self,
        cmd_buffer: &CommandBufferHandle,
        readback: &RenderHandle<ReadbackBufferPool>,
    ) {
        cmd_buffer
            .cmd_resource_barrier(
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
            )
            .unwrap();

        cmd_buffer.cmd_copy_buffer_to_buffer(
            &self.count_buffer,
            &readback.count_buffer,
            &[BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: 4,
            }],
        );

        cmd_buffer.cmd_copy_buffer_to_buffer(
            &self.picked_buffer,
            &readback.picked_buffer,
            &[BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: 1024 * 16,
            }],
        );

        cmd_buffer
            .cmd_resource_barrier(
                &[BufferBarrier {
                    buffer: &self.picked_buffer,
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::UNORDERED_ACCESS,
                    queue_transition: BarrierQueueTransition::None,
                }],
                &[],
            )
            .unwrap();
    }
}
