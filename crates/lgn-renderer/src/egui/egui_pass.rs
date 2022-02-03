use std::sync::Arc;

use lgn_graphics_api::prelude::*;
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::cgen;
use crate::components::RenderSurface;
use crate::egui::egui_plugin::Egui;

use crate::hl_gfx_api::HLCommandBuffer;

use crate::tmp_shader_data::egui_shader_family;

use crate::RenderContext;

use crate::resources::ShaderManager;

pub struct EguiPass {
    pipeline: Pipeline,
    texture_data: Option<(u64, Texture, TextureView)>,
    sampler: Sampler,
}

impl EguiPass {
    pub fn new(device_context: &DeviceContext, shader_manager: &ShaderManager) -> Self {
        let root_signature = cgen::pipeline_layout::EguiPipelineLayout::root_signature();

        let shader = shader_manager.get_shader(CGenShaderKey::make(
            egui_shader_family::ID,
            egui_shader_family::TOTO,
        ));

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
                },
                VertexLayoutAttribute {
                    format: Format::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    byte_offset: 8,
                },
                VertexLayoutAttribute {
                    format: Format::R32G32B32A32_SFLOAT,
                    buffer_index: 0,
                    location: 2,
                    byte_offset: 16,
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
                root_signature,
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
            if version == egui_ctx.font_image().version {
                return;
            }
        }

        let egui_font_image = &egui_ctx.font_image();

        let texture_def = TextureDef {
            extents: Extents3D {
                width: egui_font_image.width as u32,
                height: egui_font_image.height as u32,
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

        let egui_font_image = Arc::clone(&egui_ctx.font_image());
        let pixels = egui_font_image
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

        self.texture_data = Some((egui_font_image.version, texture, texture_view));
    }

    pub fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
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

        let transient_allocator = render_context.transient_buffer_allocator();

        cmd_buffer.bind_pipeline(&self.pipeline);

        let clipped_meshes = egui.ctx.tessellate(egui.shapes.clone());

        let mut descriptor_set = cgen::descriptor_set::EguiDescriptorSet::default();
        descriptor_set.set_font_texture(&self.texture_data.as_ref().unwrap().2);
        descriptor_set.set_font_sampler(&self.sampler);

        let descriptor_set_handle = render_context.write_descriptor_set(&descriptor_set);
        cmd_buffer.bind_descriptor_set_handle(descriptor_set_handle);

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
                transient_allocator.copy_data_slice(&vertex_data, ResourceUsage::AS_VERTEX_BUFFER);

            cmd_buffer.bind_buffer_suballocation_as_vertex_buffer(0, &sub_allocation);

            let sub_allocation =
                transient_allocator.copy_data_slice(&mesh.indices, ResourceUsage::AS_INDEX_BUFFER);

            cmd_buffer
                .bind_buffer_suballocation_as_index_buffer(&sub_allocation, IndexType::Uint32);

            let scale = 1.0;
            let mut push_constant_data = cgen::cgen_type::EguiPushConstantData::default();
            push_constant_data.set_scale(Vec2::new(scale, scale).into());
            push_constant_data.set_translation(Vec2::new(0.0, 0.0).into());
            push_constant_data.set_width(
                (render_surface.extents().width() as f32 / egui.ctx.pixels_per_point()).into(),
            );
            push_constant_data.set_height(
                (render_surface.extents().height() as f32 / egui.ctx.pixels_per_point()).into(),
            );

            cmd_buffer.push_constant(&push_constant_data);
            cmd_buffer.draw_indexed(mesh.indices.len() as u32, 0, 0);
        }
    }
}
