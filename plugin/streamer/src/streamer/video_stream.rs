use bytes::Bytes;
use legion_ecs::prelude::*;

use legion_graphics_api::{
    AddressMode, BlendState, CmdCopyTextureParams, ColorClearValue, ColorRenderTargetBinding,
    CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, CullMode, DefaultApi, DepthState,
    DescriptorDef, DescriptorElements, DescriptorKey, DescriptorSetArray, DescriptorSetArrayDef,
    DescriptorSetLayoutDef, DescriptorUpdate, DeviceContext, Extents3D, FilterType, Format, GfxApi,
    GraphicsPipelineDef, LoadOp, MemoryUsage, MipMapMode, Offset3D, PipelineType,
    PrimitiveTopology, Queue, RasterizerState, ResourceFlags, ResourceState, ResourceUsage,
    RootSignatureDef, SampleCount, SamplerDef, ShaderPackage, ShaderStageDef, ShaderStageFlags,
    StoreOp, Texture, TextureBarrier, TextureDef, TextureTiling, TextureViewDef, VertexLayout,
    MAX_DESCRIPTOR_SET_LAYOUTS,
};
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use log::{debug, warn};
use std::{cmp::min, io::Cursor, sync::Arc};

use webrtc::data::data_channel::RTCDataChannel;

use legion_codec_api::{
    backends::openh264::encoder::{self, Encoder},
    formats::{self, RBGYUVConverter},
};
use legion_mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Stream};
use legion_renderer::{components::RenderSurface, Renderer};
use legion_telemetry::prelude::*;
use legion_utils::memory::write_any;
use serde::Serialize;

fn record_frame_time_metric(microseconds: u64) {
    trace_scope!();
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Video Stream Frame Time",
        unit: "us",
    };
    record_int_metric(&FRAME_TIME_METRIC, microseconds);
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn new(mut width: u32, mut height: u32) -> Self {
        // Ensure a minimum size for the resolution.
        if width < 16 {
            width = 16;
        }

        if height < 16 {
            height = 16;
        }

        Self {
            // Make sure width & height always are multiple of 2.
            width: width & !1,
            height: height & !1,
        }
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

#[derive(Component)]
#[component(storage = "Table")]
pub struct VideoStream {
    video_data_channel: Arc<RTCDataChannel>,
    frame_id: i32,
    render_frame_count: u32,
    resolution: Resolution,
    encoder: VideoStreamEncoder,
    render_images: Vec<<DefaultApi as GfxApi>::Texture>,
    render_image_rtvs: Vec<<DefaultApi as GfxApi>::TextureView>,
    copy_images: Vec<<DefaultApi as GfxApi>::Texture>,
    cmd_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    cmd_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,
    root_signature: <DefaultApi as GfxApi>::RootSignature,
    pipeline: <DefaultApi as GfxApi>::Pipeline,
    descriptor_set_arrays: Vec<<DefaultApi as GfxApi>::DescriptorSetArray>,
    bilinear_sampler: <DefaultApi as GfxApi>::Sampler,
}

impl VideoStream {
    pub fn new(
        renderer: &Renderer,
        resolution: Resolution,
        video_data_channel: Arc<RTCDataChannel>,
    ) -> anyhow::Result<Self> {
        trace_scope!();

        let encoder = VideoStreamEncoder::new(resolution)?;
        let device_context = renderer.device_context();

        //
        // Immutable resources
        //
        let shader_compiler = HlslCompiler::new().unwrap();

        let shader_source =
            String::from_utf8(include_bytes!("../data/display_mapper.hlsl").to_vec())?;

        let shader_build_result = shader_compiler.compile(&CompileParams {
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
        })?;

        let vert_shader_module = device_context.create_shader_module(
            ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                .module_def(),
        )?;

        let frag_shader_module = device_context.create_shader_module(
            ShaderPackage::SpirV(shader_build_result.spirv_binaries[1].bytecode.clone())
                .module_def(),
        )?;

        let shader = device_context.create_shader(
            vec![
                ShaderStageDef {
                    entry_point: "main_vs".to_owned(),
                    shader_stage: ShaderStageFlags::VERTEX,
                    shader_module: vert_shader_module,
                    // reflection: shader_build_result.reflection_info.clone().unwrap(),
                },
                ShaderStageDef {
                    entry_point: "main_ps".to_owned(),
                    shader_stage: ShaderStageFlags::FRAGMENT,
                    shader_module: frag_shader_module,
                    // reflection: shader_build_result.reflection_info.clone().unwrap(),
                },
            ],
            &shader_build_result.pipeline_reflection,
        )?;

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
            descriptor_set_layouts,
            push_constant_def: None,
        };

        let root_signature = device_context.create_root_signature(&root_signature_def)?;

        let pipeline = device_context.create_graphics_pipeline(&GraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &VertexLayout::default(),
            blend_state: &BlendState::default(),
            depth_state: &DepthState::default(),
            rasterizer_state: &RasterizerState {
                cull_mode: CullMode::Back,
                ..RasterizerState::default()
            },
            primitive_topology: PrimitiveTopology::TriangleList,
            color_formats: &[Format::R8G8B8A8_UNORM],
            depth_stencil_format: None,
            sample_count: SampleCount::SampleCount1,
        })?;

        let sampler_def = SamplerDef {
            min_filter: FilterType::Linear,
            mag_filter: FilterType::Linear,
            mip_map_mode: MipMapMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            ..SamplerDef::default()
        };
        let bilinear_sampler = device_context.create_sampler(&sampler_def)?;

        //
        // Frame dependant resources
        //
        let render_frame_count = 2;
        let graphics_queue = renderer.graphics_queue();
        let mut render_images = Vec::with_capacity(render_frame_count);
        let mut render_image_rtvs = Vec::with_capacity(render_frame_count);
        let mut copy_images = Vec::with_capacity(render_frame_count);
        let mut cmd_pools = Vec::with_capacity(render_frame_count);
        let mut cmd_buffers = Vec::with_capacity(render_frame_count);

        for _ in 0..render_frame_count {
            let render_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: resolution.width,
                    height: resolution.height,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
                mem_usage: MemoryUsage::GpuOnly,
                usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Optimal,
            })?;

            let render_image_rtv = render_image.create_view(
                &TextureViewDef::as_render_target_view(render_image.texture_def()),
            )?;

            let copy_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: resolution.width,
                    height: resolution.height,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
                mem_usage: MemoryUsage::GpuToCpu,
                usage_flags: ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                tiling: TextureTiling::Linear,
            })?;

            let cmd_pool =
                graphics_queue.create_command_pool(&CommandPoolDef { transient: true })?;

            let cmd_buffer = cmd_pool.create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })?;

            render_images.push(render_image);
            render_image_rtvs.push(render_image_rtv);
            copy_images.push(copy_image);
            cmd_pools.push(cmd_pool);
            cmd_buffers.push(cmd_buffer);
        }

        let mut descriptor_set_arrays = Vec::new();
        for descriptor_set_layout in &root_signature_def.descriptor_set_layouts {
            let descriptor_set_array = device_context
                .create_descriptor_set_array(&DescriptorSetArrayDef {
                    descriptor_set_layout,
                    array_length: render_frame_count,
                })
                .unwrap();
            descriptor_set_arrays.push(descriptor_set_array);
        }

        Ok(Self {
            video_data_channel,
            frame_id: 0,
            render_frame_count: render_frame_count as u32,
            resolution,
            encoder,
            render_images,
            render_image_rtvs,
            copy_images,
            cmd_pools,
            cmd_buffers,
            root_signature,
            pipeline,
            descriptor_set_arrays,
            bilinear_sampler,
        })
    }

    pub(crate) fn resize(
        &mut self,
        renderer: &Renderer,
        resolution: Resolution,
    ) -> anyhow::Result<()> {
        trace_scope!();

        if resolution != self.resolution {
            let device_context = renderer.device_context();
            let render_frame_count = self.render_frame_count as usize;
            let mut render_images = Vec::with_capacity(render_frame_count);
            let mut render_image_rtvs = Vec::with_capacity(render_frame_count);
            let mut copy_images = Vec::with_capacity(render_frame_count);

            for _ in 0..render_frame_count {
                let render_image = device_context.create_texture(&TextureDef {
                    extents: Extents3D {
                        width: resolution.width,
                        height: resolution.height,
                        depth: 1,
                    },
                    array_length: 1,
                    mip_count: 1,
                    format: Format::R8G8B8A8_UNORM,
                    mem_usage: MemoryUsage::GpuOnly,
                    usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_TRANSFERABLE,
                    resource_flags: ResourceFlags::empty(),
                    tiling: TextureTiling::Optimal,
                })?;

                let render_image_rtv = render_image.create_view(
                    &TextureViewDef::as_render_target_view(render_image.texture_def()),
                )?;

                let copy_image = device_context.create_texture(&TextureDef {
                    extents: Extents3D {
                        width: resolution.width,
                        height: resolution.height,
                        depth: 1,
                    },
                    array_length: 1,
                    mip_count: 1,
                    format: Format::R8G8B8A8_UNORM,
                    mem_usage: MemoryUsage::GpuToCpu,
                    usage_flags: ResourceUsage::AS_TRANSFERABLE,
                    resource_flags: ResourceFlags::empty(),
                    tiling: TextureTiling::Linear,
                })?;

                render_images.push(render_image);
                render_image_rtvs.push(render_image_rtv);
                copy_images.push(copy_image);
            }

            self.resolution = resolution;
            self.render_images = render_images;
            self.render_image_rtvs = render_image_rtvs;
            self.copy_images = copy_images;

            // TODO: Fix this: this is probably bad but I wrote that just to test it.
            // self.renderer = Renderer::new(self.resolution.width, self.resolution.height);
            // self.renderer = Renderer::new();
            // self.render_surface.resize(renderer, self.resolution.width, self.resolution.height);
            self.encoder = VideoStreamEncoder::new(self.resolution)?;
        }

        Ok(())
    }

    fn record_frame_id_metric(&self) {
        static FRAME_ID_RENDERED: MetricDesc = MetricDesc {
            name: "Frame ID begin render",
            unit: "",
        };
        record_int_metric(&FRAME_ID_RENDERED, self.frame_id as u64);
    }

    pub(crate) fn render(
        &mut self,
        graphics_queue: &<DefaultApi as GfxApi>::Queue,
        wait_sem: &<DefaultApi as GfxApi>::Semaphore,
        render_surface: &mut RenderSurface,
    ) -> impl std::future::Future<Output = ()> + 'static {
        trace_scope!();
        self.record_frame_id_metric();
        let now = tokio::time::Instant::now();

        //
        // Render
        //
        {
            let render_frame_idx = 0;
            let cmd_pool = &self.cmd_pools[render_frame_idx];
            let cmd_buffer = &self.cmd_buffers[render_frame_idx];
            let render_texture = &self.render_images[render_frame_idx];
            let render_texture_rtv = &self.render_image_rtvs[render_frame_idx];
            let copy_texture = &self.copy_images[render_frame_idx];

            cmd_pool.reset_command_pool().unwrap();
            cmd_buffer.begin().unwrap();

            //
            // RenderPass
            //

            render_surface.transition_to(cmd_buffer, ResourceState::SHADER_RESOURCE);

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
                        texture_view: render_texture_rtv,
                        load_op: LoadOp::DontCare,
                        store_op: StoreOp::Store,
                        clear_value: ColorClearValue::default(),
                    }],
                    None,
                )
                .unwrap();

            cmd_buffer.cmd_bind_pipeline(&self.pipeline).unwrap();

            self.descriptor_set_arrays[0]
                .update_descriptor_set(&[
                    DescriptorUpdate {
                        array_index: render_frame_idx as u32,
                        descriptor_key: DescriptorKey::Name("hdr_sampler"),
                        elements: DescriptorElements {
                            samplers: Some(&[&self.bilinear_sampler]),
                            ..DescriptorElements::default()
                        },
                        ..DescriptorUpdate::default()
                    },
                    DescriptorUpdate {
                        array_index: render_frame_idx as u32,
                        descriptor_key: DescriptorKey::Name("hdr_image"),
                        elements: DescriptorElements {
                            texture_views: Some(&[render_surface.shader_resource_view()]),
                            ..DescriptorElements::default()
                        },
                        ..DescriptorUpdate::default()
                    },
                ])
                .unwrap();

            cmd_buffer
                .cmd_bind_descriptor_set(
                    &self.root_signature,
                    &self.descriptor_set_arrays[0],
                    (render_frame_idx) as _,
                )
                .unwrap();

            cmd_buffer.cmd_draw(3, 0).unwrap();

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

            //
            // Copy
            //

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        copy_texture,
                        ResourceState::COMMON,
                        ResourceState::COPY_DST,
                    )],
                )
                .unwrap();

            cmd_buffer
                .cmd_copy_image(
                    render_texture,
                    copy_texture,
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
                            width: self.resolution.width,
                            height: self.resolution.height,
                            depth: 1,
                        },
                    },
                )
                .unwrap();

            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        copy_texture,
                        ResourceState::COPY_DST,
                        ResourceState::COMMON,
                    )],
                )
                .unwrap();
            cmd_buffer.end().unwrap();

            //
            // Present the image
            //

            graphics_queue
                .submit(&[cmd_buffer], &[wait_sem], &[], None)
                .unwrap();

            graphics_queue.wait_for_queue_idle().unwrap();

            let sub_resource = copy_texture.map_texture().unwrap();
            self.encoder
                .converter
                .convert_rgba(sub_resource.data, sub_resource.row_pitch as usize);
            copy_texture.unmap_texture().unwrap();
        }

        // self.elapsed_secs += delta_secs * self.speed;

        // self.renderer.render(
        //     self.frame_id as usize,
        //     self.elapsed_secs,
        //     self.color.0 .0,
        //     &mut self.encoder.converter,
        // );

        let chunks = self.encoder.encode(self.frame_id);

        let elapsed = now.elapsed().as_micros() as u64;
        record_frame_time_metric(elapsed);
        let max_frame_time: u64 = 16_000;

        if elapsed >= max_frame_time {
            warn!(
                "stream: frame {:?} took {}ms",
                self.frame_id,
                elapsed / 1000
            );
        }

        let video_data_channel = Arc::clone(&self.video_data_channel);
        let frame_id = self.frame_id;

        self.frame_id += 1;

        async move {
            for (i, data) in chunks.iter().enumerate() {
                #[allow(clippy::redundant_else)]
                if let Err(err) = video_data_channel.send(data).await {
                    warn!(
                        "Failed to send frame {}-{} ({} bytes): streaming will stop: {}",
                        frame_id,
                        i,
                        data.len(),
                        err.to_string(),
                    );

                    return;
                } else {
                    debug!("Sent frame {}-{} ({} bytes).", frame_id, i, data.len());
                }
            }
        }
    }
}

struct VideoStreamEncoder {
    encoder: Encoder,
    converter: RBGYUVConverter,
    resolution: Resolution,
    mp4: Mp4Stream,
    writer: Cursor<Vec<u8>>,
    write_index: bool,
}

impl VideoStreamEncoder {
    fn new(resolution: Resolution) -> anyhow::Result<Self> {
        trace_scope!();
        let config = encoder::EncoderConfig::new(resolution.width, resolution.height)
            .constant_sps(true)
            .max_fps(60.0)
            .skip_frame(false)
            .bitrate_bps(8_000_000);

        let encoder = encoder::Encoder::with_config(config)?;

        let converter =
            formats::RBGYUVConverter::new(resolution.width as usize, resolution.height as usize);

        let mut writer = Cursor::new(Vec::<u8>::new());
        let mp4 = Mp4Stream::write_start(
            &Mp4Config {
                major_brand: b"mp42".into(),
                minor_version: 0,
                compatible_brands: vec![b"mp42".into(), b"isom".into()],
                timescale: 1000,
            },
            60,
            &mut writer,
        )?;

        Ok(Self {
            encoder,
            converter,
            resolution,
            mp4,
            writer,
            write_index: true,
        })
    }

    fn encode(&mut self, frame_id: i32) -> Vec<Bytes> {
        trace_scope!();
        self.encoder.force_intra_frame(true);
        let stream = self.encoder.encode(&self.converter).unwrap();

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
                if self.write_index {
                    self.mp4
                        .write_index(
                            &MediaConfig::AvcConfig(AvcConfig {
                                width: self.resolution.width.try_into().unwrap(),
                                height: self.resolution.height.try_into().unwrap(),
                                seq_param_set: sps.into(),
                                pic_param_set: pps.into(),
                            })
                            .into(),
                            &mut self.writer,
                        )
                        .unwrap();
                    self.write_index = false;
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

                self.mp4
                    .write_sample(
                        stream.frame_type == encoder::FrameType::IDR,
                        &vec,
                        &mut self.writer,
                    )
                    .unwrap();
            }
        }

        let chunks = split_frame_in_chunks(self.writer.get_ref(), frame_id);
        self.writer.get_mut().clear();
        self.writer.set_position(0);
        chunks
    }
}

#[derive(Serialize)]
struct ChunkHeader {
    pub chunk_index_in_frame: u8,
    pub frame_id: i32,
}

fn split_frame_in_chunks(data: &[u8], frame_id: i32) -> Vec<Bytes> {
    let max_chunk_size = 65536;
    let mut chunks = vec![];
    chunks.reserve((data.len() / max_chunk_size) + 2);

    let mut current_chunk: Vec<u8> = vec![];
    current_chunk.reserve(max_chunk_size);

    let mut current_data_index = 0;
    let mut chunk_index_in_frame: u8 = 0;
    while current_data_index < data.len() {
        current_chunk.clear();
        let header = serde_json::to_string(&ChunkHeader {
            chunk_index_in_frame,
            frame_id,
        })
        .unwrap();
        let header_payload_len: u16 = header.len() as u16;
        let header_size = std::mem::size_of::<u16>() as u16 + header_payload_len;
        write_any(&mut current_chunk, &header_payload_len);
        current_chunk.extend_from_slice(header.as_bytes());
        let end_chunk = min(
            current_data_index + max_chunk_size - header_size as usize,
            data.len(),
        );
        let chunk_data_slice = &data[current_data_index..end_chunk];
        current_chunk.extend_from_slice(chunk_data_slice);
        chunks.push(bytes::Bytes::copy_from_slice(&current_chunk));
        current_data_index = end_chunk;
        chunk_index_in_frame += 1;
    }

    chunks
}
