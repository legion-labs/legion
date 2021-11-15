use std::num::NonZeroU32;

use anyhow::Result;

use graphics_api::{
    backends::vulkan::VulkanApi, prelude::*, DefaultApi, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use legion_ecs::prelude::Query;
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};

use crate::components::RenderSurface;
pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: u32,
    num_render_frames: u32,
    frame_signal_sems: Vec<Semaphore>,
    frame_fences: Vec<Fence>,
    graphics_queue: Queue,
    command_pools: Vec<CommandPool>,
    command_buffers: Vec<CommandBuffer>,
    transient_descriptor_heaps: Vec<DescriptorHeapDrc>,

    // This should be last, as it must be destroyed last.
    api: VulkanApi,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        #![allow(unsafe_code)]
        let num_render_frames = 2u32;
        let api = unsafe { VulkanApi::new(&ApiDef::default()).unwrap() };
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

    pub fn device_context(&self) -> &DeviceContextDrc {
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

    pub fn transient_descriptor_heap(&self) -> &DescriptorHeapDrc {
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

    pub(crate) fn update(&mut self, q_render_surfaces: &mut Query<'_, '_, &mut RenderSurface>) {
        let cmd_buffer = self.get_cmd_buffer();

        for mut render_surface in q_render_surfaces.iter_mut() {
            render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

            {
                let render_pass = &render_surface.test_renderpass;
                render_pass.render(self, &render_surface, cmd_buffer);
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
    vertex_buffers: Vec<BufferDrc>,
    uniform_buffers: Vec<BufferDrc>,
    uniform_buffer_cbvs: Vec<BufferViewDrc>,
    root_signature: RootSignatureDrc,
    pipeline: PipelineDrc,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        let num_render_frames = renderer.num_render_frames;
        let mut vertex_buffers = Vec::with_capacity(num_render_frames as usize);
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
                format: Format::R32G32_SFLOAT,
                buffer_index: 0,
                location: 0,
                byte_offset: 0,
                gl_attribute_name: Some("pos".to_owned()),
            }],
            buffers: vec![VertexLayoutBuffer {
                stride: 8,
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
            let vertex_data = [0f32; 15];

            let vertex_buffer = device_context
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))
                .unwrap();
            vertex_buffer
                .copy_to_host_visible_buffer(&vertex_data)
                .unwrap();

            let uniform_data = [0f32; 4];

            let uniform_buffer = device_context
                .create_buffer(&BufferDef::for_staging_uniform_buffer_data(&uniform_data))
                .unwrap();
            uniform_buffer
                .copy_to_host_visible_buffer(&uniform_data)
                .unwrap();

            let view_def = BufferViewDef::as_const_buffer(uniform_buffer.definition());
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def).unwrap();

            vertex_buffers.push(vertex_buffer);
            uniform_buffer_cbvs.push(uniform_buffer_cbv);
            uniform_buffers.push(uniform_buffer);
        }

        Self {
            vertex_buffers,
            uniform_buffers,
            uniform_buffer_cbvs,
            root_signature,
            pipeline,
            color: [0f32, 0f32, 0f32, 1.0f32],
            speed: 1.0f32,
        }
    }

    pub fn render(
        &self,
        renderer: &Renderer,
        render_surface: &RenderSurface,
        cmd_buffer: &CommandBuffer,
    ) {
        let render_frame_idx = renderer.render_frame_idx;
        let elapsed_secs = self.speed * renderer.frame_idx as f32 / 60.0;

        //
        // Update vertices
        //
        let vertex_data = [
            0.0f32,
            0.5,
            0.5 - (elapsed_secs.cos() / 2. + 0.5),
            -0.5,
            -0.5 + (elapsed_secs.cos() / 2. + 0.5),
            -0.5,
        ];
        let vertex_buffer = &self.vertex_buffers[render_frame_idx as usize];
        vertex_buffer
            .copy_to_host_visible_buffer(&vertex_data)
            .unwrap();

        //
        // Update vertex color
        //

        let uniform_data = [1.0f32, 0.0, 0.0, 1.0];
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

        cmd_buffer
            .cmd_bind_vertex_buffers(
                0,
                &[VertexBufferBinding {
                    buffer: vertex_buffer,
                    byte_offset: 0,
                }],
            )
            .unwrap();

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

        let push_constant_data = [1.0f32, 1.0, 1.0, 1.0];
        cmd_buffer
            .cmd_push_constants(&self.root_signature, &push_constant_data)
            .unwrap();

        cmd_buffer.cmd_draw(3, 0).unwrap();

        cmd_buffer.cmd_end_render_pass().unwrap();
    }
}
