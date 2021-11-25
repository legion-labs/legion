use crate::components::RenderSurface;
use crate::Renderer;
use graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use legion_egui::Egui;
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use std::num::NonZeroU32;

pub struct EguiPass {
    vertex_buffers: Vec<Buffer>,
    index_buffers: Vec<Buffer>,
    root_signature: RootSignature,
    pipeline: Pipeline,
    descriptor_set_handle: DescriptorSetHandle,
    frequency: u32,
}

impl EguiPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer, egui_ctx: &egui::CtxRef) -> Self {
        let device_context = renderer.device_context();
        const BUFFER_SIZE: usize = 1024;
        let mut vertex_buffers = Vec::with_capacity(renderer.num_render_frames as usize);
        let mut index_buffers = Vec::with_capacity(renderer.num_render_frames as usize);
        for _ in 0..renderer.num_render_frames {
            let vertex_buffer = renderer
                .device_context()
                .create_buffer(&BufferDef::for_staging_vertex_buffer(BUFFER_SIZE))
                .unwrap();
            let index_buffer = renderer
                .device_context()
                .create_buffer(&BufferDef::for_staging_index_buffer(BUFFER_SIZE))
                .unwrap();
            vertex_buffers.push(vertex_buffer);
            index_buffers.push(index_buffer);
        }

        //
        // Shaders
        //
        let shader_compiler = HlslCompiler::new().unwrap();

        let shader_source =
            String::from_utf8(include_bytes!("../shaders/ui.hlsl").to_vec()).unwrap();

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
                    format: Format::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    byte_offset: 0,
                    gl_attribute_name: Some("pos".to_owned()),
                },
                VertexLayoutAttribute {
                    format: Format::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    byte_offset: 8,
                    gl_attribute_name: Some("uv".to_owned()),
                },
                VertexLayoutAttribute {
                    format: Format::R32G32B32A32_SFLOAT,
                    buffer_index: 0,
                    location: 2,
                    byte_offset: 16,
                    gl_attribute_name: Some("color".to_owned()),
                },
            ],
            buffers: vec![VertexLayoutBuffer {
                stride: 32,
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

        // Texture data retrieved from egui context is only valid after the call to CtrRef::run()

        let egui_texture = egui_ctx.texture();
        let staging_buffer = renderer
            .device_context()
            .create_buffer(&BufferDef::for_staging_buffer_data(
                &egui_texture.pixels,
                ResourceUsage::AS_SHADER_RESOURCE,
            ))
            .unwrap();

        let device_context = renderer.device_context();
        let texture_def = TextureDef {
            extents: Extents3D {
                width: egui_texture.width as u32,
                height: egui_texture.height as u32,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UINT,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };
        let texture = device_context.create_texture(&texture_def).unwrap();
        renderer
            .get_cmd_buffer()
            .cmd_copy_buffer_to_texture(
                &staging_buffer,
                &texture,
                &CmdCopyBufferToTextureParams::default(),
            )
            .unwrap();

        //renderer
        //    .get_cmd_buffer()
        //    .cmd_resource_barrier(
        //        &[],
        //        &[TextureBarrier::state_transition(
        //            &texture,
        //            ResourceState::COPY_DST,
        //            ResourceState::SHADER_RESOURCE,
        //        )],
        //    )
        //    .unwrap();

        let texture_view = texture
            .create_view(&TextureViewDef::as_shader_resource_view(&texture_def))
            .unwrap();

        // Create sampler
        let sampler_def = SamplerDef {
            min_filter: FilterType::Linear,
            mag_filter: FilterType::Linear,
            mip_map_mode: MipMapMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            compare_op: CompareOp::LessOrEqual,
        };
        let sampler = device_context.create_sampler(&sampler_def).unwrap();

        let heap = renderer.transient_descriptor_heap();
        let frequency = pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0]
            .definition()
            .frequency;
        let mut descriptor_set_writer = heap
            .allocate_descriptor_set(
                &pipeline
                    .root_signature()
                    .definition()
                    .descriptor_set_layouts[0],
            )
            .unwrap();
        descriptor_set_writer
            .set_descriptors(
                "font_texture",
                0,
                &[DescriptorRef::TextureView(&texture_view)],
            )
            .unwrap();
        descriptor_set_writer
            .set_descriptors("font_sampler", 0, &[DescriptorRef::Sampler(&sampler)])
            .unwrap();
        let descriptor_set_handle = descriptor_set_writer.flush(renderer.device_context());

        Self {
            vertex_buffers,
            index_buffers,
            root_signature,
            pipeline,
            descriptor_set_handle,
            frequency,
        }
    }

    pub fn render(
        &self,
        renderer: &Renderer,
        render_surface: &RenderSurface,
        cmd_buffer: &CommandBuffer,
        egui_ctx: &mut egui::CtxRef,
    ) {
        cmd_buffer
            .cmd_bind_descriptor_set_handle(
                &self.root_signature,
                self.frequency,
                self.descriptor_set_handle,
            )
            .unwrap();
        let raw_input = egui::RawInput::default();
        egui::Window::new("Test window").show(&egui_ctx, |ui| {
            ui.label("Hello, world!");
        });
        let (output, shapes) = egui_ctx.end_frame();
        let clipped_meshes = egui_ctx.tessellate(shapes);
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes {
            let vertex_data: Vec<f32> = mesh
                .vertices
                .iter()
                .flat_map(|v| {
                    let mut color = v
                        .color
                        .to_array()
                        .into_iter()
                        .map(|x| x as f32)
                        .collect::<Vec<f32>>();
                    let mut vertex = vec![v.pos.x, v.pos.y, v.uv.x, v.uv.y];
                    vertex.append(&mut color);
                    vertex
                })
                .collect();

            let vertex_buffer = renderer
                .device_context()
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))
                .unwrap();

            vertex_buffer
                .copy_to_host_visible_buffer(&vertex_data)
                .unwrap();

            let index_data = mesh.indices;
            let index_buffer = renderer
                .device_context()
                .create_buffer(&BufferDef::for_staging_index_buffer_data(&index_data))
                .unwrap();

            index_buffer
                .copy_to_host_visible_buffer(&index_data)
                .unwrap();

            cmd_buffer
                .cmd_bind_vertex_buffers(
                    0,
                    &[VertexBufferBinding {
                        buffer: &vertex_buffer,
                        byte_offset: 0,
                    }],
                )
                .unwrap();

            cmd_buffer.cmd_bind_index_buffer(&IndexBufferBinding {
                buffer: &index_buffer,
                byte_offset: 0,
                index_type: IndexType::Uint32,
            });

            let mut push_constant_data: [f32; 4] = [1.0, 1.0, 0.0, 0.0];

            cmd_buffer
                .cmd_push_constants(&self.root_signature, &push_constant_data)
                .unwrap();

            cmd_buffer
                .cmd_draw((vertex_data.len() / 8) as u32, 0)
                .unwrap();
        }
    }
}
