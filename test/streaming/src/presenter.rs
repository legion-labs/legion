use graphics_api::prelude::*;
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .init()
        .unwrap();

    run().unwrap();
}

#[cfg(target_os = "linux")]
fn run() -> GfxResult<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn run() -> GfxResult<()> {
    use presenter::window::*;
    use pso_compiler::{CompileParams, HLSLCompiler, ShaderSource};
    const WINDOW_WIDTH: u32 = 900;
    const WINDOW_HEIGHT: u32 = 600;

    //
    // Init a window
    //
    let monitors = Window::list_monitors();
    let window = Window::new(WindowType::Main(WindowMode::Windowed(WindowLocation {
        monitor: monitors[0],
        x: 0,
        y: 0,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    })));

    //
    // Create the api. GPU programming is fundamentally unsafe, so all rafx APIs should be
    // considered unsafe. However, rafx APIs are only gated by unsafe if they can cause undefined
    // behavior on the CPU for reasons other than interacting with the GPU.
    //
    #[allow(unsafe_code)]
    let mut api = unsafe {
        DefaultApi::new(
            Some(&window.native_handle()),
            &Default::default(),
            &Default::default(),
        )?
    };

    // Wrap all of this so that it gets dropped before we drop the API object. This ensures a nice
    // clean shutdown.
    {
        // A cloneable device handle, these are lightweight and can be passed across threads
        let device_context = api.device_context();

        //
        // Create a swapchain
        //
        let (window_width, window_height) = (WINDOW_WIDTH, WINDOW_HEIGHT);
        let swapchain = device_context.create_swapchain(
            &window.native_handle(),
            &SwapchainDef {
                width: window_width,
                height: window_height,
                enable_vsync: true,
            },
        )?;

        //
        // Wrap the swapchain in this helper to cut down on boilerplate. This helper is
        // multithreaded-rendering friendly! The PresentableFrame it returns can be sent to another
        // thread and presented from there, and any errors are returned back to the main thread
        // when the next image is acquired. The helper also ensures that the swapchain is rebuilt
        // as necessary.
        //
        let mut swapchain_helper =
            presenter::window::SwapchainHelper::<DefaultApi>::new(device_context, swapchain, None)?;

        //
        // Allocate a graphics queue. By default, there is just one graphics queue and it is shared.
        // There currently is no API for customizing this but the code would be easy to adapt to act
        // differently. Most recommendations I've seen are to just use one graphics queue. (The
        // rendering hardware is shared among them)
        //
        let graphics_queue = device_context.create_queue(QueueType::Graphics)?;
        let graphics_queue_cloned = graphics_queue.clone();

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
        let mut command_pools = Vec::with_capacity(swapchain_helper.image_count());
        let mut command_buffers = Vec::with_capacity(swapchain_helper.image_count());

        for _ in 0..swapchain_helper.image_count() {
            let command_pool =
                graphics_queue.create_command_pool(&CommandPoolDef { transient: true })?;

            let command_buffer = command_pool.create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })?;

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
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
        let compiler = HLSLCompiler::new().map_err(|e| e.to_string())?;

        let legion_folder = std::env::current_dir()?;
        let test_data_folder = legion_folder.join("test/test_data/shaders/");
        let test_hlsl_file = test_data_folder.join("test.hlsl");

        let compile_params = CompileParams {
            shader_source: ShaderSource::Path(&test_hlsl_file),
            entry_point: "main_vs",
            target_profile: "vs_6_1",
            defines: Vec::new(),
        };

        let vs_out = compiler
            .compile(&compile_params)
            .map_err(|e| e.to_string())?;

        let compile_params = CompileParams {
            shader_source: ShaderSource::Path(&test_hlsl_file),
            entry_point: "main_ps",
            target_profile: "ps_6_1",
            defines: Vec::new(),
        };

        let ps_out = compiler
            .compile(&compile_params)
            .map_err(|e| e.to_string())?;

        let vert_shader_package = ShaderPackage::SpirV(vs_out.bytecode);

        let frag_shader_package = ShaderPackage::SpirV(ps_out.bytecode);

        /*
            let vert_shader_package =
                ShaderPackage::SpirV(include_bytes!("shaders/shader.vert.spv").to_vec());

            let frag_shader_package =
                ShaderPackage::SpirV(include_bytes!("shaders/shader.frag.spv").to_vec());
        */
        let vert_shader_module =
            device_context.create_shader_module(vert_shader_package.module_def())?;

        let frag_shader_module =
            device_context.create_shader_module(frag_shader_package.module_def())?;

        //
        // Create the shader object by combining the stages
        //
        // Hardcode the reflecton data required to interact with the shaders. This can be generated
        // offline and loaded with the shader but this is not currently provided in rafx-api itself.
        // (But see the shader pipeline in higher-level rafx crates for example usage, generated
        // from spirv_cross)
        //
        // let color_shader_resource = ShaderResource {
        //     name: Some("color".to_string()),
        //     set_index: 0,
        //     binding: 0,
        //     shader_resource_type: ShaderResourceType::ConstantBuffer( ConstantBufferInfo{ size : 0}  ),
        //     ..Default::default()
        // };

        let vert_shader_stage_def = ShaderStageDef {
            shader_module: vert_shader_module,
            reflection: vs_out.refl_info.unwrap(),
        };

        let frag_shader_stage_def = ShaderStageDef {
            shader_module: frag_shader_module,
            reflection: ps_out.refl_info.unwrap(),
        };

        //
        // Combine the shader stages into a single shader
        //
        let shader =
            device_context.create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])?;

        //
        // TMP: TMP_extract_root_signature_def will create the root signature definition.
        // In the process, it will instanciate the DescriptorSetLayout(s).
        //
        let root_signature_def = graphics_api::backends::tmp_extract_root_signature_def(
            device_context,
            &[shader.clone()],
        )?;

        //
        // TMP: Get a ref on the DescriptorSetLayout
        //
        let descriptor_set_layout = root_signature_def.descriptor_set_layouts[0]
            .as_ref()
            .unwrap();

        //
        // Create the root signature object - it represents the pipeline layout and can be shared among
        // shaders. But one per shader is fine.
        //
        let root_signature = device_context.create_root_signature(&root_signature_def)?;

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
        let mut vertex_buffers = Vec::with_capacity(swapchain_helper.image_count());
        let mut uniform_buffers = Vec::with_capacity(swapchain_helper.image_count());
        let mut uniform_buffer_cbvs = Vec::with_capacity(swapchain_helper.image_count());

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct VertexColor_SB {
            foo: [f32; 3],
            color: [f32; 4],
            bar: f32,
        }

        for _ in 0..swapchain_helper.image_count() {
            #[rustfmt::skip]
            let vertex_data = [
                0.0f32, 0.5, 1.0, 0.0, 0.0,
                -0.5, -0.5, 0.0, 1.0, 0.0,
                0.5, 0.5, 0.0, 0.0, 1.0,
            ];

            let uniform_data = [VertexColor_SB {
                foo: [0.0, 1.0, 2.0],
                color: [1.0f32, 0.0, 1.0, 1.0],
                bar: 7f32,
            }];

            let vertex_buffer = device_context
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))?;
            vertex_buffer.copy_to_host_visible_buffer(&vertex_data)?;

            let uniform_buffer =
                device_context.create_buffer(&BufferDef::for_staging_buffer_data(
                    &uniform_data,
                    ResourceUsage::HAS_CONST_BUFFER_VIEW | ResourceUsage::HAS_SHADER_RESOURCE_VIEW,
                ))?;
            uniform_buffer.copy_to_host_visible_buffer(&uniform_data)?;

            let view_def = BufferViewDef::as_structured_buffer(
                uniform_buffer.buffer_def(),
                std::mem::size_of::<VertexColor_SB>() as u64,
                true,
            );
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def)?;

            vertex_buffers.push(vertex_buffer);
            uniform_buffers.push(uniform_buffer);
            uniform_buffer_cbvs.push(uniform_buffer_cbv);
        }

        //
        // Create texture
        //

        // let texture_def = TextureDef::default();
        // let texture_2d = device_context.create_texture(&texture_def)?;
        // drop(texture_2d);

        //
        // Descriptors are allocated in blocks and never freed. Normally you will want to build a
        // pooling system around this. (Higher-level rafx crates provide this.) But they're small
        // and cheap. We need one per swapchain image.
        //
        let mut descriptor_set_array =
            device_context.create_descriptor_set_array(&DescriptorSetArrayDef {
                descriptor_set_layout,
                array_length: 3, // One per swapchain image.
            })?;

        // Initialize them all at once here.. this can be done per-frame as well.
        #[allow(clippy::needless_range_loop)]
        for i in 0..swapchain_helper.image_count() {
            descriptor_set_array.update_descriptor_set(&[DescriptorUpdate {
                array_index: i as u32,
                descriptor_key: DescriptorKey::Name("vertex_color"),
                elements: DescriptorElements {
                    buffer_views: Some(&[&uniform_buffer_cbvs[i]]),
                    ..Default::default()
                },
                ..Default::default()
            }])?;
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

        let pipeline = device_context.create_graphics_pipeline(&GraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &vertex_layout,
            blend_state: &Default::default(),
            depth_state: &Default::default(),
            rasterizer_state: &Default::default(),
            color_formats: &[swapchain_helper.format()],
            sample_count: SampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: PrimitiveTopology::TriangleList,
        })?;

        let start_time = std::time::Instant::now();

        log::info!("Starting window event loop");
        window.event_loop(move || {
            let elapsed_seconds = start_time.elapsed().as_secs_f32();

            //
            // Acquire swapchain image
            //
            let (window_width, window_height) = (WINDOW_WIDTH, WINDOW_HEIGHT);
            let presentable_frame = swapchain_helper
                .acquire_next_image(window_width, window_height, None)
                .unwrap();
            let swapchain_texture = presentable_frame.swapchain_texture();

            let swapchain_rtv = presentable_frame.swapchain_rtv();

            //
            // Use the command pool/buffer assigned to this frame
            //
            let cmd_pool = &mut command_pools[presentable_frame.rotating_frame_index()];
            let cmd_buffer = &command_buffers[presentable_frame.rotating_frame_index()];
            let vertex_buffer = &vertex_buffers[presentable_frame.rotating_frame_index()];
            let uniform_buffer = &uniform_buffers[presentable_frame.rotating_frame_index()];

            //
            // Update the buffers
            //
            {
                #[rustfmt::skip]
                let vertex_data = [
                    0.0f32, 0.5, 1.0, 0.0, 0.0,
                    0.5 - (elapsed_seconds.cos() / 2. + 0.5), -0.5, 0.0, 1.0, 0.0,
                    -0.5 + (elapsed_seconds.cos() / 2. + 0.5), -0.5, 0.0, 0.0, 1.0,
                ];

                let color = (elapsed_seconds.cos() + 1.0) / 2.0;
                let uniform_data = [VertexColor_SB {
                    foo: [0.0, 1.0, 2.0],
                    color: [color, 0.0, 1.0 - color, 1.0],
                    bar: 7f32,
                }];

                vertex_buffer
                    .copy_to_host_visible_buffer(&vertex_data)
                    .unwrap();
                uniform_buffer
                    .copy_to_host_visible_buffer(&uniform_data)
                    .unwrap();
            }

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
                        swapchain_texture,
                        ResourceState::PRESENT,
                        ResourceState::RENDER_TARGET,
                    )],
                )
                .unwrap();

            cmd_buffer
                .cmd_begin_render_pass(
                    &[ColorRenderTargetBinding {
                        texture_view: swapchain_rtv,
                        load_op: LoadOp::Clear,
                        store_op: StoreOp::Store,
                        // array_slice: None,
                        // mip_slice: None,
                        clear_value: ColorClearValue([0.2, 0.2, 0.2, 1.0]),
                        // resolve_target: None,
                        // resolve_store_op: StoreOp::DontCare,
                        // resolve_mip_slice: None,
                        // resolve_array_slice: None,
                    }],
                    None,
                )
                .unwrap();

            cmd_buffer.cmd_bind_pipeline(&pipeline).unwrap();

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
                    &root_signature,
                    &descriptor_set_array,
                    presentable_frame.rotating_frame_index() as u32,
                )
                .unwrap();

            {
                let color = (elapsed_seconds.cos() + 1.0) / 2.0;
                let uniform_data = [VertexColor_SB {
                    foo: [0.0, 1.0, 2.0],
                    color: [color, 0.0, 1.0 - color, 1.0],
                    bar: 7f32,
                }];

                cmd_buffer
                    .cmd_push_constants(&root_signature, &uniform_data)
                    .unwrap();
            }

            cmd_buffer.cmd_draw(3, 0).unwrap();

            // Put it into a layout where we can present it

            cmd_buffer.cmd_end_render_pass().unwrap();

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        swapchain_texture,
                        ResourceState::RENDER_TARGET,
                        ResourceState::PRESENT,
                    )],
                )
                .unwrap();
            cmd_buffer.end().unwrap();

            //
            // Present the image
            //
            presentable_frame
                .present(&graphics_queue, &[cmd_buffer])
                .unwrap();
        });

        // Wait for all GPU work to complete before destroying resources it is using
        graphics_queue_cloned.wait_for_queue_idle()?;
    }

    // Optional, but calling this verifies that all rafx objects/device contexts have been
    // destroyed and where they were created. Good for finding unintended leaks!
    api.destroy()?;

    Ok(())
}
