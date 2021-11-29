use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};

use anyhow::Result;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::components::{RenderSurface, RenderSurfaceExtents, StaticMesh};
use crate::static_mesh_render_data::StaticMeshRenderData;
use crate::RenderContext;
use graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use legion_math::{Mat4, Vec3};
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use legion_transform::components::Transform;

pub struct RendererHandle<T> {
    inner: Option<Box<T>>,
}

impl<T> RendererHandle<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Some(Box::new(data)),
        }
    }

    // pub fn destroy(self) {
    //     match &self.inner {
    //         Some(e) => drop(e),
    //         None => unreachable!(),
    //     }
    //     std::mem::forget(self);
    // }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn take(&mut self) -> Self {
        Self {
            inner: self.inner.take(),
        }
    }
}

impl<T> Deref for RendererHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            Some(e) => e.as_ref(),
            None => unreachable!(),
        }
    }
}

impl<T> DerefMut for RendererHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            Some(e) => e.as_mut(),
            None => unreachable!(),
        }
    }
}

// impl<T> Drop for RendererHandle<T> {
//     fn drop(&mut self) {
//         match &self.inner {
//             Some(_) => unreachable!("todo"),
//             None => (),
//         }
//     }
// }

impl<T: Rotate> Rotate for RendererHandle<T> {
    fn rotate(&mut self) {
        match &mut self.inner {
            Some(e) => e.rotate(),
            None => unreachable!(),
        }
    }
}

pub type CommandBufferHandle = RendererHandle<CommandBuffer>;

pub struct CommandBufferPool {
    command_pool: CommandPool,
    availables: Vec<CommandBufferHandle>,
    in_flights: Vec<CommandBufferHandle>,
}

impl CommandBufferPool {
    fn new(queue: &Queue) -> Self {
        Self {
            command_pool: queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap(),
            availables: Vec::new(),
            in_flights: Vec::new(),
        }
    }

    // fn destroy(&mut self) {
    //     for cmd_buffer_handle in self.availables.drain(..) {
    //         cmd_buffer_handle.destroy();
    //     }
    //     for cmd_buffer_handle in self.in_flights.drain(..) {
    //         cmd_buffer_handle.destroy();
    //     }
    // }

    pub fn reset(&mut self) {
        self.command_pool.reset_command_pool().unwrap();
        self.availables.append(&mut self.in_flights);
    }

    pub fn acquire(&mut self) -> CommandBufferHandle {
        let result = if self.availables.is_empty() {
            let def = CommandBufferDef {
                is_secondary: false,
            };
            CommandBufferHandle::new(self.command_pool.create_command_buffer(&def).unwrap())
        } else {
            self.availables.pop().unwrap()
        };
        assert!(result.is_valid());
        result
    }

    pub fn release(&mut self, handle: CommandBufferHandle) {
        assert!(handle.is_valid());
        self.in_flights.push(handle);
    }
}

impl Rotate for CommandBufferPool {
    fn rotate(&mut self) {
        self.reset();
    }
}

// impl Drop for CommandBufferPool {
//     fn drop(&mut self) {
//         self.destroy();
//     }
// }

pub type CommandBufferPoolHandle = RendererHandle<CommandBufferPool>;
pub type DescriptorHeapHandle = RendererHandle<DescriptorHeap>;

trait Rotate {
    fn rotate(&mut self);
}

struct RotatingResource<T: Rotate> {
    num_cpu_frames: usize,
    cur_cpu_frame: usize,
    available: Vec<T>,
    in_use: Vec<Vec<T>>,
}

impl<T: Rotate> RotatingResource<T> {
    fn new(num_cpu_frames: usize) -> Self {
        Self {
            num_cpu_frames,
            cur_cpu_frame: 0,
            available: Vec::new(),
            in_use: (0..num_cpu_frames).map(|_| Vec::new()).collect(),
        }
    }

    // fn destroy(&mut self) {
    //     for data in self.available.drain(..) {
    //         data.destroy();
    //     }
    //     for list in self.in_use.drain(..) {
    //         for data in list.drain(..) {
    //             data.destroy();
    //         }
    //     }
    // }

    fn rotate(&mut self) {
        let next_cpu_frame = (self.cur_cpu_frame + 1) % self.num_cpu_frames;
        self.available.append(&mut self.in_use[next_cpu_frame]);
        self.available.iter_mut().for_each(|x| x.rotate());
        self.cur_cpu_frame = next_cpu_frame;
    }

    fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> T {
        let result = if self.available.is_empty() {
            create_fn()
        } else {
            self.available.pop().unwrap()
        };
        result
    }

    fn release(&mut self, data: T) {
        self.in_use[self.cur_cpu_frame].push(data);
    }
}

// impl<T: Rotate> Drop for RotatingResource<T> {
//     fn drop(&mut self) {
//         self.destroy();
//     }
// }

impl Rotate for DescriptorHeap {
    fn rotate(&mut self) {
        self.reset().unwrap();
    }
}

pub trait Presenter: Send + Sync {
    fn resize(&mut self, extents: RenderSurfaceExtents);
    fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
    );
}

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    frame_fences: Vec<Fence>,
    graphics_queue: RwLock<Queue>,
    command_buffer_pools: RwLock<RotatingResource<CommandBufferPoolHandle>>,
    descriptor_heaps: RwLock<RotatingResource<DescriptorHeapHandle>>,
		transient_buffer: TransientPagedBuffer,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        #![allow(unsafe_code)]
        let num_render_frames = 2usize;
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();

        let transient_buffer = TransientPagedBuffer::new(device_context, 16);

        Ok(Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
                .collect(),
            graphics_queue: RwLock::new(device_context.create_queue(QueueType::Graphics).unwrap()),
            command_buffer_pools: RwLock::new(RotatingResource::new(num_render_frames)),
            descriptor_heaps: RwLock::new(RotatingResource::new(num_render_frames)),
            // presenters: RwLock::new(Vec::new()),
transient_buffer,
            api,
        })
    }

    pub fn api(&self) -> &GfxApi {
        &self.api
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

    pub fn frame_fence(&self) -> &Fence {
        &self.frame_fences[self.render_frame_idx]
    }

    pub fn queue(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            QueueType::Compute => todo!(),
            QueueType::Transfer => todo!(),
            QueueType::Decode => todo!(),
            QueueType::Encode => todo!(),
        }
    }

    // pub fn queue_mut(&self, queue_type: QueueType) -> RwLockWriteGuard<'_, Queue> {
    //     match queue_type {
    //         QueueType::Graphics => self.graphics_queue.write(),
    //         QueueType::Compute => todo!(),
    //         QueueType::Transfer => todo!(),
    //         QueueType::Decode => todo!(),
    //         QueueType::Encode => todo!(),
    //     }
    // }

    pub fn acquire_command_buffer_pool(&self, queue_type: QueueType) -> CommandBufferPoolHandle {
        let queue = self.queue(queue_type);
        let mut pool = self.command_buffer_pools.write();
        pool.acquire_or_create(|| {
            CommandBufferPoolHandle::new(CommandBufferPool::new(queue.deref()))
        })
    }

    pub fn release_command_buffer_pool(&self, handle: CommandBufferPoolHandle) {
        let mut pool = self.command_buffer_pools.write();
        pool.release(handle);
    }

    pub fn acquire_transient_descriptor_heap(
        &self,
        heap_def: &DescriptorHeapDef,
    ) -> DescriptorHeapHandle {
        let mut pool = self.descriptor_heaps.write();
        pool.acquire_or_create(|| {
            DescriptorHeapHandle::new(
                self.device_context()
                    .create_descriptor_heap(&heap_def)
                    .unwrap(),
            )
        })
    }

    pub fn release_transient_descriptor_heap(&self, handle: DescriptorHeapHandle) {
        let mut pool = self.descriptor_heaps.write();
        pool.release(handle);
    }
pub fn transient_buffer(&self) -> &TransientPagedBuffer {
        &self.transient_buffer
    }
    // pub fn frame_signal_semaphore(&self) -> &Semaphore {
    //     let render_frame_index = self.render_frame_idx;
    //     &self.frame_signal_sems[render_frame_index as usize]
    // }

    pub(crate) fn begin_frame(&mut self) {
        //
        // Update frame indices
        //
        self.frame_idx += 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;

        //
        // Store on stack
        //
        let render_frame_idx = self.render_frame_idx;

        //
        // Wait for the next frame to be available
        //
        let signal_fence = &self.frame_fences[render_frame_idx];
        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

        //
        // Now, it is safe to free memory
        //
        let device_context = self.api.device_context();
        device_context.free_gpu_memory().unwrap();

        //
        // Rotate resources
        //
        {
            let mut pool = self.command_buffer_pools.write();
            pool.rotate();
        }
        {
            let mut pool = self.descriptor_heaps.write();
            pool.rotate();
        }
        // let cmd_pool = &self.command_pools[render_frame_idx as usize];
        // let cmd_buffer = &self.command_buffers[render_frame_idx as usize];
        // let transient_descriptor_heap = &self.transient_descriptor_heaps[render_frame_idx as usize];

        // cmd_pool.reset_command_pool().unwrap();
        // cmd_buffer.begin().unwrap();

        self.transient_buffer.begin_frame(self.device_context());
    }

    // pub(crate) fn update(
    //     &mut self,
    //     q_render_surfaces: &mut Query<'_, '_, &mut RenderSurface>,
    //     query: &Query<'_, '_, (&Transform, &StaticMesh)>,
    // ) {
    //     let render_context = self.get_render_context();
    //     // let cmd_buffer = self.get_cmd_buffer();

    //     let query = query.iter().collect::<Vec<(&Transform, &StaticMesh)>>();

    //     for mut render_surface in q_render_surfaces.iter_mut() {
    //         let render_pass = &render_surface.test_renderpass;
    //         render_pass.render(&render_context, &render_surface, query.as_slice());
    //     }
    // }

    // pub(crate) fn end_frame(&mut self) {
    //     let render_frame_idx = self.render_frame_idx;
    //     // let signal_semaphore = &self.frame_signal_sems[render_frame_idx as usize];
    //     let signal_fence = &self.frame_fences[render_frame_idx as usize];
    //     // let cmd_buffer = &self.command_buffers[render_frame_idx as usize];

    //     // cmd_buffer.end().unwrap();

    //     let pool = self.acquire_command_buffer_pool(QueueType::Graphics);
    //     let cmd_buffer = pool.get();
    //     cmd_buffer.begin();
    //     cmd_buffer.end();

    //     self.graphics_queue
    //         .submit(&[cmd_buffer], &[], &[], Some(signal_fence))
    //         .unwrap();
    // }
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
        }
    }

    pub fn render(
        &self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &CommandBuffer,
        render_surface: &mut RenderSurface,
        static_meshes: &[(&Transform, &StaticMesh)],
    ) {
        //
        // Fill command buffer
        //
        render_surface.transition_to(&cmd_buffer, ResourceState::RENDER_TARGET);

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

        let color_table = [
            (1.0f32, 0.0f32, 0.0f32),
            (1.0f32, 1.0f32, 0.0f32),
            (1.0f32, 0.0f32, 1.0f32),
            (0.0f32, 0.0f32, 1.0f32),
            (0.0f32, 1.0f32, 0.0f32),
            (0.0f32, 1.0f32, 1.0f32),
        ];

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

        for (index, (transform, static_mesh_component)) in static_meshes.iter().enumerate() {
            let mesh_id = static_mesh_component.mesh_id;
            if mesh_id >= self.static_meshes.len() {
                continue;
            }

            let mesh = &self.static_meshes[static_mesh_component.mesh_id];

            let transient_allocator =
                TransientBufferAllocator::new(renderer.transient_buffer(), 1000);

            let mut sub_allocation = transient_allocator.copy_data(None, &mesh.vertices, 0);

            renderer
                .transient_buffer()
                .bind_allocation_as_vertex_buffer(cmd_buffer, &sub_allocation);

            let color = color_table[index % color_table.len()];

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

            let const_buffer_view = renderer
                .transient_buffer()
                .const_buffer_view_for_allocation(&sub_allocation);

            let mut descriptor_set_writer =
                heap.allocate_descriptor_set(descriptor_set_layout).unwrap();
            descriptor_set_writer
                .set_descriptors(
                    "uniform_data",
                    0,
                    &[DescriptorRef::BufferView(&const_buffer_view)],
                )
                .unwrap();
            let descriptor_set_handle = descriptor_set_writer.flush(renderer.device_context());

            cmd_buffer
                .cmd_bind_descriptor_set_handle(
                    &self.root_signature,
                    descriptor_set_layout.definition().frequency,
                    descriptor_set_handle,
                )
                .unwrap();

            // cmd_buffer
            //     .cmd_push_constants(
            //         &self.root_signature,
            //         &(sub_allocation.offset_of_page + sub_allocation.last_alloc_offset),
            //     )
            //     .unwrap();

            cmd_buffer
                .cmd_draw((mesh.num_vertices()) as u32, 0)
                .unwrap();
        }

        cmd_buffer.cmd_end_render_pass().unwrap();
    }
}
