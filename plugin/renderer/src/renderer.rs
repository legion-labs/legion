use graphics_api::prelude::*;
use graphics_api::backends::shared::tmp_extract_root_signature_def;

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    frame_fences: Vec<<DefaultApi as GfxApi>::Fence>,
    graphics_queue: <DefaultApi as GfxApi>::Queue,
    command_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    command_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,
    vertex_buffers: Vec<<DefaultApi as GfxApi>::Buffer>,
    uniform_buffers: Vec<<DefaultApi as GfxApi>::Buffer>,
    // render_images: Vec<<DefaultApi as GfxApi>::Texture>,
    // render_views: Vec<<DefaultApi as GfxApi>::TextureView>,
    descriptor_set_array: <DefaultApi as GfxApi>::DescriptorSetArray,
    root_signature: <DefaultApi as GfxApi>::RootSignature,
    pipeline: <DefaultApi as GfxApi>::Pipeline,

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

        let num_buffered_frames = 2;

        //
        // Allocate a graphics queue. By default, there is just one graphics queue and it is shared.
        // There currently is no API for customizing this but the code would be easy to adapt to act
        // differently. Most recommendations I've seen are to just use one graphics queue. (The
        // rendering hardware is shared among them)
        //
        let graphics_queue = device_context.create_queue(QueueType::Graphics).unwrap();

        //
        // Some default data we can render
        //
        #[rustfmt::skip]
                let vertex_data = [
                    0.0f32, 0.5, 1.0, 0.0, 0.0,
                    -0.5, -0.5, 0.0, 1.0, 0.0,
                    0.5, 0.5, 0.0, 0.0, 1.0,
                ];

        let uniform_data = [1.0f32, 0.0, 1.0, 1.0];

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
        let mut command_pools = Vec::with_capacity(num_buffered_frames);
        let mut command_buffers = Vec::with_capacity(num_buffered_frames);
        let mut vertex_buffers = Vec::with_capacity(num_buffered_frames);
        let mut uniform_buffers = Vec::with_capacity(num_buffered_frames);
        let mut uniform_buffer_cbvs = Vec::with_capacity(num_buffered_frames);
        // let mut render_images = Vec::with_capacity(num_buffered_frames);
        // let mut render_views = Vec::with_capacity(num_buffered_frames);
        let mut frame_fences = Vec::with_capacity(num_buffered_frames);
        // let mut copy_images = Vec::with_capacity(parallel_render_count);

        for _ in 0..num_buffered_frames {
            let command_pool = graphics_queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap();

            let command_buffer = command_pool
                .create_command_buffer(&CommandBufferDef {
                    is_secondary: false,
                })
                .unwrap();

            let vertex_buffer = device_context
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))
                .unwrap();
            vertex_buffer
                .copy_to_host_visible_buffer(&vertex_data)
                .unwrap();

            let uniform_buffer = device_context
                .create_buffer(&BufferDef::for_staging_uniform_buffer_data(&uniform_data))
                .unwrap();
            uniform_buffer
                .copy_to_host_visible_buffer(&uniform_data)
                .unwrap();

            let view_def = BufferViewDef::as_const_buffer(uniform_buffer.buffer_def());
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def).unwrap();

            // let render_image = device_context
            //     .create_texture(&TextureDef {
            //         extents: Extents3D {
            //             width,
            //             height,
            //             depth: 1,
            //         },
            //         array_length: 1,
            //         mip_count: 1,
            //         format: Format::R8G8B8A8_UNORM,
            //         usage_flags: ResourceUsage::HAS_SHADER_RESOURCE_VIEW
            //             | ResourceUsage::HAS_RENDER_TARGET_VIEW,
            //         resource_flags: ResourceFlags::empty(),
            //         mem_usage: MemoryUsage::GpuOnly,
            //         tiling: TextureTiling::Optimal,
            //     })
            //     .unwrap();

            // let render_view_def = TextureViewDef::as_render_target_view(render_image.texture_def());
            // let render_view = render_image.create_view(&render_view_def).unwrap();            
            let frame_fence = device_context.create_fence().unwrap();

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
            vertex_buffers.push(vertex_buffer);
            uniform_buffer_cbvs.push(uniform_buffer_cbv);
            uniform_buffers.push(uniform_buffer);
            // render_images.push(render_image);
            // render_views.push(render_view);
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

        //
        // Create the shader object by combining the stages
        //
        // Hardcode the reflecton data required to interact with the shaders. This can be generated
        // offline and loaded with the shader but this is not currently provided in rafx-api itself.
        // (But see the shader pipeline in higher-level rafx crates for example usage, generated
        // from spirv_cross)
        //
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

        //
        // Combine the shader stages into a single shader
        //
        let shader = device_context
            .create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])
            .unwrap();

        let root_signature_def = tmp_extract_root_signature_def(
            device_context,
            &[shader.clone()],
        )
        .unwrap();
        //
        // Create the root signature object - it represents the pipeline layout and can be shared among
        // shaders. But one per shader is fine.
        //
        let root_signature = device_context
            .create_root_signature(&root_signature_def)
            .unwrap();
        let descriptor_set_layout = root_signature_def.descriptor_set_layouts[0]
            .as_ref()
            .unwrap();

        //
        // Descriptors are allocated in blocks and never freed. Normally you will want to build a
        // pooling system around this. (Higher-level rafx crates provide this.) But they're small
        // and cheap. We need one per swapchain image.
        //
        let mut descriptor_set_array = device_context
            .create_descriptor_set_array(&DescriptorSetArrayDef {
                descriptor_set_layout,
                array_length: 3, // One per swapchain image.
            })
            .unwrap();

        // Initialize them all at once here.. this can be done per-frame as well.
        #[allow(clippy::needless_range_loop)]
        for i in 0..num_buffered_frames {
            descriptor_set_array
                .update_descriptor_set(&[DescriptorUpdate {
                    array_index: i as u32,
                    descriptor_key: DescriptorKey::Name("color"),
                    elements: DescriptorElements {
                        buffer_views: Some(&[&uniform_buffer_cbvs[i]]),
                        ..Default::default()
                    },
                    ..Default::default()
                }])
                .unwrap();
        }

        //
        // Now set up the pipeline. LOTS of things can be configured here, but aside from the vertex
        // layout most of it can be left as default.
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

        Renderer {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames: num_buffered_frames,
            frame_fences,
            api,
            graphics_queue,
            command_pools,
            command_buffers,
            vertex_buffers,
            uniform_buffers,
            // render_images,
            // render_views,
            descriptor_set_array,
            root_signature,
            pipeline,
        }
    }

    pub fn render(
        &mut self,        
        render_view: &<DefaultApi as GfxApi>::TextureView
    ) {
        let render_frame_idx = self.render_frame_idx;
        let elapsed_secs = self.frame_idx as f32 / 60.0;

        let signal_fence = &self.frame_fences[render_frame_idx];

        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

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

        let uniform_data = [1.0f32, 0.0, 0.0, 1.0];

        //
        // Acquire swapchain image
        //
        // let render_texture = &self.render_images[render_frame_idx];
        // let render_view = &self.render_views[render_frame_idx];

        //
        // Use the command pool/buffer assigned to this frame
        //
        let cmd_pool = &self.command_pools[render_frame_idx];
        let cmd_buffer = &self.command_buffers[render_frame_idx];
        let vertex_buffer = &self.vertex_buffers[render_frame_idx];
        let uniform_buffer = &self.uniform_buffers[render_frame_idx];

        //
        // Update the buffers
        //
        vertex_buffer
            .copy_to_host_visible_buffer(&vertex_data)
            .unwrap();
        uniform_buffer
            .copy_to_host_visible_buffer(&uniform_data)
            .unwrap();

        //
        // Record the command buffer. For now just transition it between layouts
        //
        cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();

        // Put it into a layout where we can draw on it
        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_view.texture(),
                    ResourceState::COPY_SRC,
                    ResourceState::RENDER_TARGET,
                )],
            )
            .unwrap();

        cmd_buffer
            .cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_view,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue([0.2, 0.2, 0.2, 1.0]),
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

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_view.texture(),
                    ResourceState::RENDER_TARGET,
                    ResourceState::COPY_SRC,
                )],
            )
            .unwrap();

        cmd_buffer.end().unwrap();
        
        self.graphics_queue
            .submit(&[cmd_buffer], &[], &[], Some(signal_fence))
            .unwrap();

        self.frame_idx = self.frame_idx + 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;
    }

    pub fn api(&self) -> &DefaultApi {
        &self.api
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphics_queue.wait_for_queue_idle().unwrap();
    }
}
