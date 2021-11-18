use std::num::NonZeroU32;

use anyhow::Result;

use crate::components::{RenderSurface, StaticMesh};
use crate::static_mesh_render_data::StaticMeshRenderData;
use graphics_api::{prelude::*, DefaultApi, MAX_DESCRIPTOR_SET_LAYOUTS};
use legion_ecs::prelude::Query;
use legion_math::{Mat4, Vec3};
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use legion_transform::components::Transform;
pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: u32,
    pub num_render_frames: u32,
    frame_signal_sems: Vec<Semaphore>,
    frame_fences: Vec<Fence>,
    graphics_queue: Queue,
    command_pools: Vec<CommandPool>,
    command_buffers: Vec<CommandBuffer>,
    transient_descriptor_heaps: Vec<DescriptorHeap>,

    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        #![allow(unsafe_code)]
        let num_render_frames = 2u32;
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();
        let graphics_queue = device_context.create_queue(QueueType::Graphics).unwrap();
        let mut command_pools = Vec::with_capacity(num_render_frames as usize);
        let mut command_buffers = Vec::with_capacity(num_render_frames as usize);
        let mut frame_signal_sems = Vec::with_capacity(num_render_frames as usize);
        let mut frame_fences = Vec::with_capacity(num_render_frames as usize);
        let mut transient_descriptor_heaps = Vec::with_capacity(num_render_frames as usize);

        for _ in 0..num_render_frames {
            let command_pool =
                graphics_queue.create_command_pool(&CommandPoolDef { transient: true })?;

            let command_buffer = command_pool.create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })?;

            let frame_signal_sem = device_context.create_semaphore()?;

            let frame_fence = device_context.create_fence()?;

            let transient_descriptor_heap_def = DescriptorHeapDef {
                transient: true,
                max_descriptor_sets: 10 * 1024,
                sampler_count: 256,
                constant_buffer_count: 10 * 1024,
                buffer_count: 10 * 1024,
                rw_buffer_count: 10 * 1024,
                texture_count: 10 * 1024,
                rw_texture_count: 10 * 1024,
            };
            let transient_descriptor_heap =
                device_context.create_descriptor_heap(&transient_descriptor_heap_def)?;

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
            frame_signal_sems.push(frame_signal_sem);
            frame_fences.push(frame_fence);
            transient_descriptor_heaps.push(transient_descriptor_heap);
        }

        Ok(Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_signal_sems,
            frame_fences,
            graphics_queue,
            command_pools,
            command_buffers,
            transient_descriptor_heaps,
            api,
        })
    }

    pub fn api(&self) -> &DefaultApi {
        &self.api
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.api.device_context()
    }

    pub fn graphics_queue(&self) -> &Queue {
        &self.graphics_queue
    }

    pub fn get_cmd_buffer(&self) -> &CommandBuffer {
        let render_frame_index = self.render_frame_idx;
        &self.command_buffers[render_frame_index as usize]
    }

    pub fn frame_signal_semaphore(&self) -> &Semaphore {
        let render_frame_index = self.render_frame_idx;
        &self.frame_signal_sems[render_frame_index as usize]
    }

    pub fn transient_descriptor_heap(&self) -> &DescriptorHeap {
        let render_frame_index = self.render_frame_idx;
        &self.transient_descriptor_heaps[render_frame_index as usize]
    }

    pub(crate) fn begin_frame(&mut self) {
        //
        // Update frame indices
        //
        self.frame_idx += 1;
        self.render_frame_idx = (self.frame_idx % self.num_render_frames as usize) as u32;

        //
        // Store on stack
        //
        let render_frame_idx = self.render_frame_idx;

        //
        // Wait for the next frame to be available
        //
        let signal_fence = &self.frame_fences[render_frame_idx as usize];
        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

        //
        // Now, it is safe to free memory
        //
        let device_context = self.api.device_context();
        device_context.free_gpu_memory().unwrap();

        //
        // Tmp. Reset command buffer.
        //
        let cmd_pool = &self.command_pools[render_frame_idx as usize];
        let cmd_buffer = &self.command_buffers[render_frame_idx as usize];
        let transient_descriptor_heap = &self.transient_descriptor_heaps[render_frame_idx as usize];

        cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();
        transient_descriptor_heap.reset().unwrap();
    }

    pub(crate) fn update(
        &mut self,
        q_render_surfaces: &mut Query<'_, '_, &mut RenderSurface>,
        query: &Query<'_, '_, (&Transform, &StaticMesh)>,
    ) {
        let cmd_buffer = self.get_cmd_buffer();

        let query = query.iter().collect::<Vec<(&Transform, &StaticMesh)>>();

        for mut render_surface in q_render_surfaces.iter_mut() {
            render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

            {
                let render_pass = &render_surface.test_renderpass;
                render_pass.render(self, &render_surface, cmd_buffer, query.as_slice());
            }
        }
    }

    pub(crate) fn end_frame(&mut self) {
        let render_frame_idx = self.render_frame_idx;
        let signal_semaphore = &self.frame_signal_sems[render_frame_idx as usize];
        let signal_fence = &self.frame_fences[render_frame_idx as usize];
        let cmd_buffer = &self.command_buffers[render_frame_idx as usize];

        cmd_buffer.end().unwrap();

        self.graphics_queue
            .submit(&[cmd_buffer], &[], &[signal_semaphore], Some(signal_fence))
            .unwrap();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue.wait_for_queue_idle().unwrap();
    }
}

pub struct TmpRenderPass {
    static_meshes: Vec<StaticMeshRenderData>,
    uniform_buffers: Vec<Buffer>,
    uniform_buffer_cbvs: Vec<BufferView>,
    root_signature: RootSignature,
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        let num_render_frames = renderer.num_render_frames;
        let mut uniform_buffers = Vec::with_capacity(num_render_frames as usize);
        let mut uniform_buffer_cbvs = Vec::with_capacity(num_render_frames as usize);

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
            attributes: vec![VertexLayoutAttribute {
                format: Format::R32G32B32_SFLOAT,
                buffer_index: 0,
                location: 0,
                byte_offset: 0,
                gl_attribute_name: Some("pos".to_owned()),
            }],
            buffers: vec![VertexLayoutBuffer {
                stride: 12,
                rate: VertexAttributeRate::Vertex,
            }],
        };

        let pipeline = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
                root_signature: &root_signature,
                vertex_layout: &vertex_layout,
                blend_state: &BlendState::default(),
                depth_state: &DepthState::default(),
                rasterizer_state: &RasterizerState::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        //
        // Per frame resources
        //
        for _ in 0..renderer.num_render_frames {
            let uniform_data = [0f32; 4];

            let uniform_buffer = device_context
                .create_buffer(&BufferDef::for_staging_uniform_buffer_data(&uniform_data))
                .unwrap();
            uniform_buffer
                .copy_to_host_visible_buffer(&uniform_data)
                .unwrap();

            let view_def = BufferViewDef::as_const_buffer(uniform_buffer.definition());
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def).unwrap();

            uniform_buffer_cbvs.push(uniform_buffer_cbv);
            uniform_buffers.push(uniform_buffer);
        }

        let static_meshes = vec![
            StaticMeshRenderData::new_plane(1.0, renderer),
            StaticMeshRenderData::new_cube(0.5, renderer),
            StaticMeshRenderData::new_pyramid(0.5, 1.0, renderer),
        ];

        Self {
            static_meshes,
            uniform_buffers,
            uniform_buffer_cbvs,
            root_signature,
            pipeline,
            color: [0f32, 0f32, 0.2f32, 1.0f32],
            speed: 1.0f32,
        }
    }

    pub fn render(
        &self,
        renderer: &Renderer,
        render_surface: &RenderSurface,
        cmd_buffer: &CommandBuffer,
        static_meshes: &[(&Transform, &StaticMesh)],
    ) {
        let render_frame_idx = renderer.render_frame_idx;
        //let elapsed_secs = self.speed * renderer.frame_idx as f32 / 60.0;

        //
        // Update vertex color
        //

        let uniform_data = [1.0f32, 1.0, 1.0, 1.0];
        let uniform_buffer = &self.uniform_buffers[render_frame_idx as usize];
        let uniform_buffer_cbv = &self.uniform_buffer_cbvs[render_frame_idx as usize];

        uniform_buffer
            .copy_to_host_visible_buffer(&uniform_data)
            .unwrap();

        //
        // Fill command buffer
        //

        cmd_buffer
            .cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.render_target_view(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue(self.color),
                }],
                &None,
            )
            .unwrap();

        cmd_buffer.cmd_bind_pipeline(&self.pipeline).unwrap();

        let heap = renderer.transient_descriptor_heap();
        let descriptor_set_layout = &self
            .pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0];
        let mut descriptor_set_writer =
            heap.allocate_descriptor_set(descriptor_set_layout).unwrap();
        descriptor_set_writer
            .set_descriptors(
                "uniform_data",
                0,
                &[DescriptorRef::BufferView(uniform_buffer_cbv)],
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

        let color_table = [
            (1.0f32, 0.0f32, 0.0f32),
            (1.0f32, 1.0f32, 0.0f32),
            (1.0f32, 0.0f32, 1.0f32),
            (0.0f32, 0.0f32, 1.0f32),
            (0.0f32, 1.0f32, 0.0f32),
            (0.0f32, 1.0f32, 1.0f32),
        ];

        let fov_y_radians: f32 = 45.0;
        let aspect_ratio: f32 = 1280.0 / 720.0; //TODO: Query resource
        let z_near: f32 = 0.01;
        let z_far: f32 = 100.0;
        let projection_matrix = Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

        let eye = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let view_matrix = Mat4::look_at_lh(eye, center, up);

        //let transforms = vec![
        //    Transform::from_rotation(Quat::from_euler(
        //        EulerRot::XYZ,
        //        elapsed_secs.cos() * std::f32::consts::PI,
        //        elapsed_secs.sin() * std::f32::consts::PI,
        //        0.0,
        //    )),
        //    Transform::from_xyz(
        //        elapsed_secs.cos() * std::f32::consts::PI * 0.25,
        //        elapsed_secs.sin() * std::f32::consts::PI * 0.25,
        //        0.0,
        //    ),
        //];

        for (index, (transform, static_mesh_component)) in static_meshes.iter().enumerate() {
            let mesh_id = static_mesh_component.mesh_id;
            if mesh_id >= self.static_meshes.len() {
                continue;
            }

            let mesh = &self.static_meshes[static_mesh_component.mesh_id];

            cmd_buffer
                .cmd_bind_vertex_buffers(
                    0,
                    &[VertexBufferBinding {
                        buffer: &mesh.vertex_buffers[render_frame_idx as usize],
                        byte_offset: 0,
                    }],
                )
                .unwrap();

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

            cmd_buffer
                .cmd_push_constants(&self.root_signature, &push_constant_data)
                .unwrap();

            cmd_buffer
                .cmd_draw((mesh.vertices.len() / 3) as u32, 0)
                .unwrap();
        }

        cmd_buffer.cmd_end_render_pass().unwrap();
    }
}
