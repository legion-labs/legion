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

fn run() -> GfxResult<()> {
    use codec_api::{backends::openh264::encoder, formats};
    use mp4::{AvcConfig, MediaConfig, Mp4Config};
    use std::io::{Cursor, Write};

    const TARGET_WIDTH: u32 = 1920;
    const TARGET_HEIGHT: u32 = 1080;

    let config = encoder::EncoderConfig::new(TARGET_WIDTH, TARGET_HEIGHT)
        .constant_sps(true)
        .max_fps(60.0)
        .skip_frame(false)
        .bitrate_bps(20_000_000);

    let mut encoder = encoder::Encoder::with_config(config).unwrap();
    let mut converter = formats::RBGYUVConverter::new(TARGET_WIDTH as _, TARGET_HEIGHT as _);

    let mut data = Cursor::new(Vec::<u8>::new());
    let mut mp4_stream = mp4::Mp4Stream::write_start(
        &Mp4Config {
            major_brand: b"mp42".into(),
            minor_version: 512,
            compatible_brands: vec![b"mp42".into(), b"isom".into()],
            timescale: 1000,
        },
        60,
        &mut data,
    )
    .unwrap();

    // Create the api. GPU programming is fundamentally unsafe, so all rafx APIs should be
    // considered unsafe. However, rafx APIs are only gated by unsafe if they can cause undefined
    // behavior on the CPU for reasons other than interacting with the GPU.
    //
    #[allow(unsafe_code)]
    let mut api = unsafe {
        DefaultApi::new(&ApiDef {
            windowing_mode: ExtensionMode::Disabled,
            ..ApiDef::default()
        })?
    };

    // Wrap all of this so that it gets dropped before we drop the API object. This ensures a nice
    // clean shutdown.
    {
        // A cloneable device handle, these are lightweight and can be passed across threads
        let device_context = api.device_context();

        let parallel_render_count = 2;

        //
        // Allocate a graphics queue. By default, there is just one graphics queue and it is shared.
        // There currently is no API for customizing this but the code would be easy to adapt to act
        // differently. Most recommendations I've seen are to just use one graphics queue. (The
        // rendering hardware is shared among them)
        //
        let graphics_queue = device_context.create_queue(QueueType::Graphics)?;
        let graphics_queue_cloned = graphics_queue.clone();

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
        let mut command_pools = Vec::with_capacity(parallel_render_count);
        let mut command_buffers = Vec::with_capacity(parallel_render_count);
        let mut vertex_buffers = Vec::with_capacity(parallel_render_count);
        let mut uniform_buffers = Vec::with_capacity(parallel_render_count);
        let mut uniform_buffer_cbvs = Vec::with_capacity(parallel_render_count);
        let mut render_images = Vec::with_capacity(parallel_render_count);
        let mut render_views = Vec::with_capacity(parallel_render_count);
        let mut copy_images = Vec::with_capacity(parallel_render_count);

        let mut file_h264 = std::fs::File::create("D:/test.h264").unwrap();
        for _ in 0..parallel_render_count {
            let command_pool =
                graphics_queue.create_command_pool(&CommandPoolDef { transient: true })?;

            let command_buffer = command_pool.create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })?;

            let vertex_buffer = device_context
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))?;
            vertex_buffer.copy_to_host_visible_buffer(&vertex_data)?;

            let uniform_buffer = device_context
                .create_buffer(&BufferDef::for_staging_uniform_buffer_data(&uniform_data))?;
            uniform_buffer.copy_to_host_visible_buffer(&uniform_data)?;

            let view_def = BufferViewDef::as_const_buffer(uniform_buffer.buffer_def());
            let uniform_buffer_cbv = uniform_buffer.create_view(&view_def)?;

            let render_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: TARGET_WIDTH,
                    height: TARGET_HEIGHT,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                // sample_count: SampleCount::SampleCount1,
                format: Format::R8G8B8A8_UNORM,
                usage_flags: ResourceUsage::HAS_SHADER_RESOURCE_VIEW
                    | ResourceUsage::HAS_RENDER_TARGET_VIEW,
                resource_flags: ResourceFlags::empty(),
                mem_usage: MemoryUsage::GpuOnly,
                // dimensions: TextureDimensions::Dim2D,
                tiling: TextureTiling::Optimal,
            })?;

            let render_view = render_image.create_view(&TextureViewDef::as_render_target_view(
                render_image.texture_def(),
            ))?;

            let copy_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: TARGET_WIDTH,
                    height: TARGET_HEIGHT,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
                mem_usage: MemoryUsage::GpuToCpu,
                usage_flags: ResourceUsage::HAS_SHADER_RESOURCE_VIEW,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Linear,
            })?;

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
            vertex_buffers.push(vertex_buffer);
            uniform_buffers.push(uniform_buffer);
            uniform_buffer_cbvs.push(uniform_buffer_cbv);
            render_images.push(render_image);
            render_views.push(render_view);
            copy_images.push(copy_image);
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
            ShaderPackage::SpirV(include_bytes!("shaders/shader.vert.spv").to_vec());

        let frag_shader_package =
            ShaderPackage::SpirV(include_bytes!("shaders/shader.frag.spv").to_vec());

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

        let color_shader_resource = ShaderResource {
            name: "color".to_owned(),
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
        let shader =
            device_context.create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])?;

        let root_signature_def = graphics_api::backends::tmp_extract_root_signature_def(
            device_context,
            &[shader.clone()],
        )?;

        //
        // Create the root signature object - it represents the pipeline layout and can be shared among
        // shaders. But one per shader is fine.
        //
        let root_signature = device_context.create_root_signature(&root_signature_def)?;
        let descriptor_set_layout = root_signature_def.descriptor_set_layouts[0]
            .as_ref()
            .unwrap();

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
        for i in 0..parallel_render_count {
            descriptor_set_array.update_descriptor_set(&[DescriptorUpdate {
                array_index: i as u32,
                descriptor_key: DescriptorKey::Name("color"),
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
            color_formats: &[Format::R8G8B8A8_UNORM],
            sample_count: SampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: PrimitiveTopology::TriangleList,
        })?;

        let start_time = std::time::Instant::now();

        log::info!("Starting window event loop");
        let mut sps_pps_written = false;
        for i in 0..300 {
            let elapsed_seconds = start_time.elapsed().as_secs_f32();

            #[rustfmt::skip]
            let vertex_data = [
                0.0f32, 0.5, 1.0, 0.0, 0.0,
                0.5 - (elapsed_seconds.cos() / 2. + 0.5), -0.5, 0.0, 1.0, 0.0,
                -0.5 + (elapsed_seconds.cos() / 2. + 0.5), -0.5, 0.0, 0.0, 1.0,
            ];

            let color = (elapsed_seconds.cos() + 1.0) / 2.0;
            let uniform_data = [color, 0.0, 1.0 - color, 1.0];

            //
            // Acquire swapchain image
            //
            let render_texture = &render_images[i % 2];
            let render_view = &render_views[i % 2];

            //
            // Use the command pool/buffer assigned to this frame
            //
            let cmd_pool = &mut command_pools[i % 2];
            let cmd_buffer = &command_buffers[i % 2];
            let vertex_buffer = &vertex_buffers[i % 2];
            let uniform_buffer = &uniform_buffers[i % 2];

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
                        render_texture,
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
                .cmd_bind_descriptor_set(&root_signature, &descriptor_set_array, (i % 2) as _)
                .unwrap();
            cmd_buffer.cmd_draw(3, 0).unwrap();

            // Put it into a layout where we can present it

            cmd_buffer.cmd_end_render_pass().unwrap();

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        render_texture,
                        ResourceState::RENDER_TARGET,
                        ResourceState::COPY_SRC,
                    )],
                )
                .unwrap();

            let dst_texture = &copy_images[i % 2];
            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        dst_texture,
                        ResourceState::UNDEFINED,
                        ResourceState::COPY_DST,
                    )],
                )
                .unwrap();

            cmd_buffer.cmd_copy_image(
                render_texture,
                dst_texture,
                &CmdCopyTextureParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offset: Offset3D { x: 0, y: 0, z: 0 },
                    dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                    src_mip_level: 0,
                    dst_mip_level: 0,
                    src_array_slice: 0,
                    dst_array_slice: 0,
                    extent: Extents3D {
                        width: TARGET_WIDTH,
                        height: TARGET_HEIGHT,
                        depth: 1,
                    },
                },
            )?;
            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        dst_texture,
                        ResourceState::COPY_DST,
                        ResourceState::COMMON,
                    )],
                )
                .unwrap();
            cmd_buffer.end().unwrap();
            //
            // Present the image
            //

            graphics_queue.submit(&[cmd_buffer], &[], &[], None)?;
            graphics_queue.wait_for_queue_idle()?;

            let sub_resource = dst_texture.map_texture()?;
            converter.convert_rgba(sub_resource.data, sub_resource.row_pitch as usize);

            encoder.force_intra_frame(true);
            let stream = encoder.encode(&converter).unwrap();

            file_h264.write_all(&stream.write_vec()).unwrap();
            for layer in &stream.layers {
                if !layer.is_video {
                    let mut sps: &[u8] = &[];
                    let mut pps: &[u8] = &[];
                    for nalu in &layer.nal_units {
                        if nalu[4] == 103 {
                            sps = &nalu[4..];
                        } else if nalu[4] == 104 {
                            pps = &nalu[4..];
                        }
                    }
                    if !sps_pps_written {
                        mp4_stream
                            .write_index(
                                &MediaConfig::AvcConfig(AvcConfig {
                                    width: TARGET_WIDTH.try_into().unwrap(),
                                    height: TARGET_HEIGHT.try_into().unwrap(),
                                    seq_param_set: sps.into(),
                                    pic_param_set: pps.into(),
                                })
                                .into(),
                                &mut data,
                            )
                            .unwrap();
                        sps_pps_written = true;
                    }
                    continue;
                }
                for nalu in &layer.nal_units {
                    let size = nalu.len() - 4;
                    let mut vec = vec![];
                    vec.extend_from_slice(nalu);
                    vec[0] = (size >> 24) as u8;
                    vec[1] = ((size >> 16) & 0xFF) as u8;
                    vec[2] = ((size >> 8) & 0xFF) as u8;
                    vec[3] = (size & 0xFF) as u8;

                    mp4_stream
                        .write_sample(
                            stream.frame_type == encoder::FrameType::IDR,
                            &vec,
                            &mut data,
                        )
                        .unwrap();
                }
            }
            dst_texture.unmap_texture()?;
        }

        // Wait for all GPU work to complete before destroying resources it is using
        graphics_queue_cloned.wait_for_queue_idle()?;
    }
    std::fs::write("D:/test.mp4", data.into_inner()).unwrap();

    // Optional, but calling this verifies that all rafx objects/device contexts have been
    // destroyed and where they were created. Good for finding unintended leaks!
    api.destroy()?;

    Ok(())
}
