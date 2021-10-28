use graphics_api::backends::shared::tmp_extract_root_signature_def;
use graphics_api::prelude::*;

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    frame_signal_sems: Vec<<DefaultApi as GfxApi>::Semaphore>,
    frame_fences: Vec<<DefaultApi as GfxApi>::Fence>,
    graphics_queue: <DefaultApi as GfxApi>::Queue,
    command_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    command_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,

    // This should be last, as it must be destroyed last.
    api: DefaultApi,
}

impl Renderer {
    pub fn new() -> Renderer {
        #[allow(unsafe_code)]
        let api = unsafe { DefaultApi::new(&ApiDef::default()).unwrap() };

        // Wrap all of this so that it gets dropped before we drop the API object. This ensures a nice
        // clean shutdown.

        // A cloneable device handle, these are lightweight and can be passed across threads
        let device_context = api.device_context();

        let num_render_frames = 2;

        //
        // Allocate a graphics queue. By default, there is just one graphics queue and it is shared.
        // There currently is no API for customizing this but the code would be easy to adapt to act
        // differently. Most recommendations I've seen are to just use one graphics queue. (The
        // rendering hardware is shared among them)
        //
        let graphics_queue = device_context.create_queue(QueueType::Graphics).unwrap();

        //
        // Create command pools/command buffers. The command pools need to be immutable while they are
        // being processed by a queue, so create one per swapchain image.
        //
        // Create vertex buffers (with position/color information) and a uniform buffers that we
        // can bind to pass additional info.
        //
        // In this demo, the color data in the shader is pulled from
        // the uniform instead of the vertex buffer. Buffers also need to be immutable while
        // processed, so we need one per swapchain image
        //
        let mut command_pools = Vec::with_capacity(num_render_frames);
        let mut command_buffers = Vec::with_capacity(num_render_frames);
        let mut frame_signal_sems = Vec::with_capacity(num_render_frames);
        let mut frame_fences = Vec::with_capacity(num_render_frames);

        for _ in 0..num_render_frames {
            let command_pool = graphics_queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap();

            let command_buffer = command_pool
                .create_command_buffer(&CommandBufferDef {
                    is_secondary: false,
                })
                .unwrap();

            let frame_signal_sem = device_context.create_semaphore().unwrap();

            let frame_fence = device_context.create_fence().unwrap();

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
            frame_signal_sems.push(frame_signal_sem);
            frame_fences.push(frame_fence);
        }

        //
        // Load a shader from source - this part is API-specific. vulkan will want SPV, metal wants
        // source code or even better a pre-compiled library. But the metal compiler toolchain only
        // works on mac/windows and is a command line tool without programmatic access.
        //
        // In an engine, it would be better to pack different formats depending on the platform
        // being built. Higher level rafx crates can help with this. But this is meant as a simple
        // example without needing those crates.
        //
        // ShaderPackage holds all the data needed to create a GPU shader module object. It is
        // heavy-weight, fully owning the data. We create by loading files from disk. This object
        // can be stored as an opaque, binary object and loaded directly if you prefer.
        //
        // ShaderModuleDef is a lightweight reference to this data. Here we create it from the
        // ShaderPackage, but you can create it yourself if you already loaded the data in some
        // other way.
        //
        // The resulting shader modules represent a loaded shader GPU object that is used to create
        // shaders. Shader modules can be discarded once the graphics pipeline is built.
        //

        //
        // Some default data we can render
        //

        Renderer {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            frame_signal_sems,
            frame_fences,
            graphics_queue,
            command_pools,
            command_buffers,
            api,
        }
    }

    pub fn api(&self) -> &DefaultApi {
        &self.api
    }

    pub fn device_context(&self) -> &<DefaultApi as GfxApi>::DeviceContext {
        self.api.device_context()
    }

    pub fn graphics_queue(&self) -> &<DefaultApi as GfxApi>::Queue {
        &self.graphics_queue
    }

    pub fn get_cmd_buffer(&self) -> &<DefaultApi as GfxApi>::CommandBuffer {
        let render_frame_index = self.render_frame_idx;
        &self.command_buffers[render_frame_index]
    }

    pub fn frame_signal_semaphore(&self) -> &<DefaultApi as GfxApi>::Semaphore {
        let render_frame_index = self.render_frame_idx;
        &self.frame_signal_sems[render_frame_index]
    }

    pub fn begin_frame(&self) {
        let render_frame_idx = self.render_frame_idx;
        let signal_fence = &self.frame_fences[render_frame_idx];

        //
        // Wait for the next frame to be available
        //
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
        let cmd_pool = &self.command_pools[render_frame_idx];
        let cmd_buffer = &self.command_buffers[render_frame_idx];

        cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();
    }

    pub fn end_frame(&mut self) {
        let render_frame_idx = self.render_frame_idx;
        let signal_semaphore = &self.frame_signal_sems[render_frame_idx];
        let signal_fence = &self.frame_fences[render_frame_idx];
        let cmd_buffer = &self.command_buffers[render_frame_idx];

        cmd_buffer.end().unwrap();

        self.graphics_queue
            .submit(&[cmd_buffer], &[], &[&signal_semaphore], Some(signal_fence))
            .unwrap();

        self.frame_idx = self.frame_idx + 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue.wait_for_queue_idle().unwrap();
    }
}

#[derive(Debug)]
pub struct TmpRenderPass {
    vertex_buffers: Vec<<DefaultApi as GfxApi>::Buffer>,
    uniform_buffers: Vec<<DefaultApi as GfxApi>::Buffer>,
    descriptor_set_array: <DefaultApi as GfxApi>::DescriptorSetArray,
    root_signature: <DefaultApi as GfxApi>::RootSignature,
    pipeline: <DefaultApi as GfxApi>::Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();
        let num_render_frames = renderer.num_render_frames;
        let mut vertex_buffers = Vec::with_capacity(num_render_frames);
        let mut uniform_buffers = Vec::with_capacity(num_render_frames);
        let mut uniform_buffer_cbvs = Vec::with_capacity(num_render_frames);

        //
        // Shaders
        //
        let vert_shader_package =
            ShaderPackage::SpirV(include_bytes!("../shaders/shader.vert.spv").to_vec());

        let frag_shader_package =
            ShaderPackage::SpirV(include_bytes!("../shaders/shader.frag.spv").to_vec());

        let vert_shader_module = device_context
            .create_shader_module(vert_shader_package.module_def())
            .unwrap();
        let frag_shader_module = device_context
            .create_shader_module(frag_shader_package.module_def())
            .unwrap();

        let color_shader_resource = ShaderResource {
            name: "color".to_string(),
            set_index: 0,
            binding: 0,
            shader_resource_type: ShaderResourceType::ConstantBuffer,
            element_count: 0,
            used_in_shader_stages: ShaderStageFlags::VERTEX,
        };

        let vert_shader_stage_def = ShaderStageDef {
            shader_module: vert_shader_module,
            reflection: ShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: ShaderStageFlags::VERTEX,
                compute_threads_per_group: None,
                shader_resources: vec![color_shader_resource],
                push_constants: Vec::new(),
            },
        };

        let frag_shader_stage_def = ShaderStageDef {
            shader_module: frag_shader_module,
            reflection: ShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: ShaderStageFlags::FRAGMENT,
                compute_threads_per_group: None,
                shader_resources: Vec::new(),
                push_constants: Vec::new(),
            },
        };

        let shader = device_context
            .create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])
            .unwrap();

        //
        // Root signature
        //

        let root_signature_def =
            tmp_extract_root_signature_def(device_context, &[shader.clone()]).unwrap();

        let root_signature = device_context
            .create_root_signature(&root_signature_def)
            .unwrap();
        let descriptor_set_layout = root_signature_def.descriptor_set_layouts[0]
            .as_ref()
            .unwrap();
        let mut descriptor_set_array = device_context
            .create_descriptor_set_array(&DescriptorSetArrayDef {
                descriptor_set_layout,
                array_length: 3, // One per swapchain image.
            })
            .unwrap();

        //
        // Pipeline state
        //
        let vertex_layout = VertexLayout {
            attributes: vec![
                VertexLayoutAttribute {
                    format: Format::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    byte_offset: 0,
                    gl_attribute_name: Some("pos".to_string()),
                },
                VertexLayoutAttribute {
                    format: Format::R32G32B32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    byte_offset: 8,
                    gl_attribute_name: Some("in_color".to_string()),
                },
            ],
            buffers: vec![VertexLayoutBuffer {
                stride: 20,
                rate: VertexAttributeRate::Vertex,
            }],
        };

        let pipeline = device_context
            .create_graphics_pipeline(&GraphicsPipelineDef {
                shader: &shader,
                root_signature: &root_signature,
                vertex_layout: &vertex_layout,
                blend_state: &Default::default(),
                depth_state: &Default::default(),
                rasterizer_state: &Default::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        //
        // Per frame resources
        //
        for i in 0..renderer.num_render_frames {
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

            let view_def = BufferViewDef::as_const_buffer(uniform_buffer.buffer_def());
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def).unwrap();

            descriptor_set_array
                .update_descriptor_set(&[DescriptorUpdate {
                    array_index: i as u32,
                    descriptor_key: DescriptorKey::Name("color"),
                    elements: DescriptorElements {
                        buffer_views: Some(&[&uniform_buffer_cbv]),
                        ..Default::default()
                    },
                    ..Default::default()
                }])
                .unwrap();

            vertex_buffers.push(vertex_buffer);
            uniform_buffer_cbvs.push(uniform_buffer_cbv);
            uniform_buffers.push(uniform_buffer);
        }

        Self {
            vertex_buffers,
            uniform_buffers,
            descriptor_set_array,
            root_signature,
            pipeline,
            color: [0f32, 0f32, 0f32, 1.0f32],
            speed: 1.0f32,
        }
    }

    pub fn render(
        &self,
        renderer: &Renderer,
        cmd_buffer: &<DefaultApi as GfxApi>::CommandBuffer,
        render_view: &<DefaultApi as GfxApi>::TextureView,
    ) {
        let render_frame_idx = renderer.render_frame_idx;
        let elapsed_secs = self.speed * renderer.frame_idx as f32 / 60.0;

        //
        // Update vertices
        //
        let vertex_data = [
            0.0f32,
            0.5,
            1.0,
            0.0,
            0.0,
            0.5 - (elapsed_secs.cos() / 2. + 0.5),
            -0.5,
            0.0,
            1.0,
            0.0,
            -0.5 + (elapsed_secs.cos() / 2. + 0.5),
            -0.5,
            0.0,
            0.0,
            1.0,
        ];
        let vertex_buffer = &self.vertex_buffers[render_frame_idx];
        vertex_buffer
            .copy_to_host_visible_buffer(&vertex_data)
            .unwrap();

        //
        // Update vertex color
        //

        let uniform_data = [1.0f32, 0.0, 0.0, 1.0];
        let uniform_buffer = &self.uniform_buffers[render_frame_idx];

        uniform_buffer
            .copy_to_host_visible_buffer(&uniform_data)
            .unwrap();

        //
        // Fill command buffer
        //

        cmd_buffer
            .cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_view,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue(self.color),
                }],
                None,
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
        cmd_buffer
            .cmd_bind_descriptor_set(
                &self.root_signature,
                &self.descriptor_set_array,
                (render_frame_idx) as _,
            )
            .unwrap();
        cmd_buffer.cmd_draw(3, 0).unwrap();

        // Put it into a layout where we can present it

        cmd_buffer.cmd_end_render_pass().unwrap();
    }
}

impl Drop for TmpRenderPass {
    fn drop(&mut self) {}
}
