#![allow(unsafe_code)]

use std::num::NonZeroU32;

use anyhow::Result;
use lgn_graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_graphics_utils::TransientPagedBuffer;
use lgn_math::{Mat4, Vec3};
use lgn_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use lgn_transform::components::Transform;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::components::{RenderSurface, StaticMesh};
use crate::memory::{BumpAllocator, BumpAllocatorHandle};
use crate::resources::{
    CommandBufferPool, CommandBufferPoolHandle, CpuPool, DescriptorPool, DescriptorPoolHandle,
    GpuSafePool,
};
use crate::static_mesh_render_data::StaticMeshRenderData;
use crate::RenderContext;
use crate::EguiPass;
use legion_egui::Egui;

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    frame_fences: Vec<Fence>,
    graphics_queue: RwLock<Queue>,
    command_buffer_pools: RwLock<GpuSafePool<CommandBufferPool>>,
    descriptor_pools: RwLock<GpuSafePool<DescriptorPool>>,
    transient_buffer: TransientPagedBuffer,
    bump_allocator_pool: RwLock<CpuPool<BumpAllocator>>,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

unsafe impl Send for Renderer {}

unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        #![allow(unsafe_code)]
        let num_render_frames = 2usize;
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();

        Ok(Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
                .collect(),
            graphics_queue: RwLock::new(device_context.create_queue(QueueType::Graphics).unwrap()),
            command_buffer_pools: RwLock::new(GpuSafePool::new(num_render_frames)),
            descriptor_pools: RwLock::new(GpuSafePool::new(num_render_frames)),
            transient_buffer: TransientPagedBuffer::new(device_context, 16),
            bump_allocator_pool: RwLock::new(CpuPool::new()),
            api,
        })
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.api.device_context()
    }

    pub fn num_render_frames(&self) -> usize {
        self.num_render_frames
    }

    pub fn render_frame_idx(&self) -> usize {
        self.render_frame_idx
    }

    pub fn queue(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            _ => unreachable!(),
        }
    }

    // TMP: change that.
    pub fn transient_buffer(&self) -> TransientPagedBuffer {
        self.transient_buffer.clone()
    }

    pub(crate) fn acquire_command_buffer_pool(
        &self,
        queue_type: QueueType,
    ) -> CommandBufferPoolHandle {
        let queue = self.queue(queue_type);
        let mut pool = self.command_buffer_pools.write();
        pool.acquire_or_create(|| CommandBufferPool::new(&*queue))
    }

    pub(crate) fn release_command_buffer_pool(&self, handle: CommandBufferPoolHandle) {
        let mut pool = self.command_buffer_pools.write();
        pool.release(handle);
    }

    pub(crate) fn acquire_descriptor_pool(
        &self,
        heap_def: &DescriptorHeapDef,
    ) -> DescriptorPoolHandle {
        let mut pool = self.descriptor_pools.write();
        pool.acquire_or_create(|| DescriptorPool::new(self.device_context(), heap_def))
    }

    pub(crate) fn release_descriptor_pool(&self, handle: DescriptorPoolHandle) {
        let mut pool = self.descriptor_pools.write();
        pool.release(handle);
    }

    pub(crate) fn acquire_bump_allocator(&self) -> BumpAllocatorHandle {
        let mut pool = self.bump_allocator_pool.write();
        pool.acquire_or_create(BumpAllocator::new)
    }

    pub(crate) fn release_bump_allocator(&self, handle: BumpAllocatorHandle) {
        let mut pool = self.bump_allocator_pool.write();
        pool.release(handle);
    }

    pub(crate) fn begin_frame(&mut self) {
        //
        // Update frame indices
        //
        self.frame_idx += 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;

        //
        // Wait for the next cpu frame to be available
        //
        let signal_fence = &self.frame_fences[self.render_frame_idx];
        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

        //
        // Now, it is safe to free memory
        //
        let device_context = self.api.device_context();
        device_context.free_gpu_memory().unwrap();

        //
        // Broadcast begin frame event
        //
        {
            let mut pool = self.command_buffer_pools.write();
            pool.begin_frame();
        }
        {
            let mut pool = self.descriptor_pools.write();
            pool.begin_frame();
        }
        {
            let mut pool = self.bump_allocator_pool.write();
            pool.begin_frame();
        }

        // TMP: todo
        self.transient_buffer.begin_frame(self.device_context());
    }

    pub(crate) fn end_frame(&mut self) {
        let graphics_queue = self.graphics_queue.write();
        let frame_fence = &self.frame_fences[self.render_frame_idx];
        graphics_queue
            .submit(&[], &[], &[], Some(frame_fence))
            .unwrap();

        //
        // Broadcast end frame event
        //
        self.transient_buffer.end_frame(&graphics_queue);

        {
            let mut pool = self.command_buffer_pools.write();
            pool.end_frame();
        }
        {
            let mut pool = self.descriptor_pools.write();
            pool.end_frame();
        }
        {
            let mut pool = self.bump_allocator_pool.write();
            pool.end_frame();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let graphics_queue = self.queue(QueueType::Graphics);
        graphics_queue.wait_for_queue_idle().unwrap();
    }
}

pub struct TmpRenderPass {
    static_meshes: Vec<StaticMeshRenderData>,
    root_signature: RootSignature,
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        //
        // Shaders
        //
        let shader_compiler = HlslCompiler::new().unwrap();

        let shader_source =
            String::from_utf8(include_bytes!("../shaders/shader.hlsl").to_vec()).unwrap();

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
            pipeline_type: PipelineType::Graphics,
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
            attributes: vec![
                VertexLayoutAttribute {
                    format: Format::R32G32B32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    byte_offset: 0,
                    gl_attribute_name: Some("pos".to_owned()),
                },
                VertexLayoutAttribute {
                    format: Format::R32G32B32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    byte_offset: 12,
                    gl_attribute_name: Some("normal".to_owned()),
                },
            ],
            buffers: vec![VertexLayoutBuffer {
                stride: 24,
                rate: VertexAttributeRate::Vertex,
            }],
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
                root_signature: &root_signature,
                vertex_layout: &vertex_layout,
                blend_state: &BlendState::default(),
                depth_state: &depth_state,
                rasterizer_state: &RasterizerState::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: Some(Format::D32_SFLOAT),
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        //
        // Per frame resources
        //
        let static_meshes = vec![
            StaticMeshRenderData::new_plane(1.0, renderer),
            StaticMeshRenderData::new_cube(0.5, renderer),
            StaticMeshRenderData::new_pyramid(0.5, 1.0, renderer),
        ];

        Self {
            static_meshes,
            root_signature,
            pipeline,
            color: [0f32, 0f32, 0.2f32, 1.0f32],
            speed: 1.0f32,
            egui_pass: EguiPass::new(renderer),
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn render(
        &self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &CommandBuffer,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&Transform, &StaticMesh)],
    ) {
        {
            let bump = render_context.acquire_bump_allocator();
            let x = bump.alloc(3);
            *x += 1;
            render_context.release_bump_allocator(bump);
        }

        render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        cmd_buffer
            .cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.render_target_view(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue(self.color),
                }],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: render_surface.depth_stencil_texture_view(),
                    depth_load_op: LoadOp::Clear,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::DontCare,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                }),
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
        let projection_matrix = Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

        let eye = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let view_matrix = Mat4::look_at_lh(eye, center, up);

        let transient_allocator = render_context.acquire_transient_buffer_allocator();

        for (_index, (transform, static_mesh_component)) in static_meshes.iter().enumerate() {
            let mesh_id = static_mesh_component.mesh_id;
            if mesh_id >= self.static_meshes.len() {
                continue;
            }

            let mesh = &self.static_meshes[static_mesh_component.mesh_id];

            let mut sub_allocation = transient_allocator.copy_data(None, &mesh.vertices, 0);

            render_context
                .renderer()
                .transient_buffer()
                .bind_allocation_as_vertex_buffer(cmd_buffer, &sub_allocation);

            let color: (f32, f32, f32, f32) = (
                f32::from(static_mesh_component.color.r) / 255.0f32,
                f32::from(static_mesh_component.color.g) / 255.0f32,
                f32::from(static_mesh_component.color.b) / 255.0f32,
                f32::from(static_mesh_component.color.a) / 255.0f32,
            );

            let world = transform.compute_matrix();
            let mut push_constant_data: [f32; 52] = [0.0; 52];
            world.write_cols_to_slice(&mut push_constant_data[0..]);
            view_matrix.write_cols_to_slice(&mut push_constant_data[16..]);
            projection_matrix.write_cols_to_slice(&mut push_constant_data[32..]);
            push_constant_data[48] = color.0;
            push_constant_data[49] = color.1;
            push_constant_data[50] = color.2;
            push_constant_data[51] = 1.0;

            sub_allocation =
                transient_allocator.copy_data(Some(sub_allocation), &push_constant_data, 64);

            let const_buffer_view = render_context
                .renderer()
                .transient_buffer()
                .const_buffer_view_for_allocation(&sub_allocation);

            let mut descriptor_set_writer =
                render_context.alloc_descriptor_set(descriptor_set_layout);

            descriptor_set_writer
                .set_descriptors(
                    "uniform_data",
                    0,
                    &[DescriptorRef::BufferView(&const_buffer_view)],
                )
                .unwrap();
            let descriptor_set_handle =
                descriptor_set_writer.flush(render_context.renderer().device_context());

            cmd_buffer
                .cmd_bind_descriptor_set_handle(
                    &self.root_signature,
                    descriptor_set_layout.definition().frequency,
                    descriptor_set_handle,
                )
                .unwrap();

            cmd_buffer
                .cmd_draw((mesh.num_vertices()) as u32, 0)
                .unwrap();
        }

        render_context.release_transient_buffer_allocator(transient_allocator);

        self.egui_pass.render(renderer, render_surface, cmd_buffer);
    }
}
