use crate::components::RenderSurface;
use crate::egui::egui_plugin::Egui;
use crate::hl_gfx_api::HLCommandBuffer;
use crate::RenderContext;
use crate::Renderer;
use lgn_graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_pso_compiler::{CompileParams, EntryPoint, ShaderSource};
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct EguiPass {
    root_signature: RootSignature,
    pipeline: Pipeline,
    texture_data: Option<(u64, Texture, TextureView)>,
    sampler: Sampler,
}

impl EguiPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        //
        // Shaders
        //
        let shader_compiler = renderer.shader_compiler();

        let shader_source =
            String::from_utf8(include_bytes!("../../shaders/ui.hlsl").to_vec()).unwrap();

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
                blend_state: &BlendState::default_alpha_enabled(),
                depth_state: &DepthState::default(),
                rasterizer_state: &RasterizerState::default(),
                color_formats: &[Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            })
            .unwrap();

        let device_context = renderer.device_context();

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

        Self {
            root_signature,
            pipeline,
            texture_data: None,
            sampler,
        }
    }

    pub fn update_font_texture(
        &mut self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &HLCommandBuffer<'_>,
        egui_ctx: &egui::CtxRef,
    ) {
        if let Some((version, ..)) = self.texture_data {
            if version == egui_ctx.texture().version {
                return;
            }
        }

        let egui_texture = &egui_ctx.texture();

        let texture_def = TextureDef {
            extents: Extents3D {
                width: egui_texture.width as u32,
                height: egui_texture.height as u32,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R32_SFLOAT,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };
        let texture = render_context
            .renderer()
            .device_context()
            .create_texture(&texture_def)
            .unwrap();

        let texture_view = texture
            .create_view(&TextureViewDef::as_shader_resource_view(&texture_def))
            .unwrap();

        let egui_texture = Arc::clone(&egui_ctx.texture());
        let pixels = egui_texture
            .pixels
            .clone()
            .into_iter()
            .map(|i| f32::from(i) / 255.0)
            .collect::<Vec<f32>>();

        let staging_buffer = render_context.renderer().device_context().create_buffer(
            &BufferDef::for_staging_buffer_data(&pixels, ResourceUsage::empty()),
        );

        let alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::CpuToGpu,
            always_mapped: true,
        };

        let buffer_memory = MemoryAllocation::from_buffer(
            render_context.renderer().device_context(),
            &staging_buffer,
            &alloc_def,
        );

        buffer_memory.copy_to_host_visible_buffer(&pixels);

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                &texture,
                ResourceState::UNDEFINED,
                ResourceState::COPY_DST,
            )],
        );

        cmd_buffer.copy_buffer_to_texture(
            &staging_buffer,
            &texture,
            &CmdCopyBufferToTextureParams::default(),
        );

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                &texture,
                ResourceState::COPY_DST,
                ResourceState::SHADER_RESOURCE,
            )],
        );

        self.texture_data = Some((egui_texture.version, texture, texture_view));
    }

    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &HLCommandBuffer<'_>,
        render_surface: &RenderSurface,
        egui: &Egui,
    ) {
        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_surface.render_target_view(),
                load_op: LoadOp::Load,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue([0.0; 4]),
            }],
            &None,
        );

        cmd_buffer.bind_pipeline(&self.pipeline);

        let descriptor_set_layout = &self
            .pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0];
        let mut descriptor_set_writer = render_context.alloc_descriptor_set(descriptor_set_layout);
        descriptor_set_writer
            .set_descriptors_by_name(
                "font_texture",
                &[DescriptorRef::TextureView(
                    &self.texture_data.as_ref().unwrap().2,
                )],
            )
            .unwrap();
        descriptor_set_writer
            .set_descriptors_by_name("font_sampler", &[DescriptorRef::Sampler(&self.sampler)])
            .unwrap();
        let descriptor_set_handle =
            descriptor_set_writer.flush(render_context.renderer().device_context());

        cmd_buffer.bind_descriptor_set_handle(
            PipelineType::Graphics,
            &self.root_signature,
            descriptor_set_layout.definition().frequency,
            descriptor_set_handle,
        );
        let clipped_meshes = egui.ctx.tessellate(egui.shapes.clone());

        let transient_allocator = render_context.transient_buffer_allocator();

        for egui::ClippedMesh(_clip_rect, mesh) in clipped_meshes {
            if mesh.is_empty() {
                continue;
            }

            let vertex_data: Vec<f32> = mesh
                .vertices
                .iter()
                .flat_map(|v| {
                    let mut color = v
                        .color
                        .to_array()
                        .into_iter()
                        .map(f32::from)
                        .collect::<Vec<f32>>();
                    let mut vertex = vec![v.pos.x, v.pos.y, v.uv.x, v.uv.y];
                    vertex.append(&mut color);
                    vertex
                })
                .collect();

            let sub_allocation =
                transient_allocator.copy_data(&vertex_data, ResourceUsage::AS_VERTEX_BUFFER);

            cmd_buffer.bind_suballocation_as_vertex_buffer(0, &sub_allocation);

            let sub_allocation =
                transient_allocator.copy_data(&mesh.indices, ResourceUsage::AS_INDEX_BUFFER);

            cmd_buffer.bind_suballocation_as_index_buffer(&sub_allocation, IndexType::Uint32);

            let scale = 1.0;
            let push_constant_data: [f32; 6] = [
                scale,
                scale,
                0.0,
                0.0,
                render_surface.extents().width() as f32 / egui.ctx.pixels_per_point(),
                render_surface.extents().height() as f32 / egui.ctx.pixels_per_point(),
            ];

            cmd_buffer.push_constants(&self.root_signature, &push_constant_data);

            cmd_buffer.draw_indexed(mesh.indices.len() as u32, 0, 0);
        }
    }
}
