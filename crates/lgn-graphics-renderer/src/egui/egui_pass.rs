use lgn_graphics_api::prelude::*;
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::cgen;
use crate::components::RenderSurface;
use crate::egui::Egui;

use crate::RenderContext;

use crate::resources::{PipelineDef, PipelineHandle, PipelineManager};

pub struct EguiPass {
    pipeline_handle: PipelineHandle,
    texture_data: Option<(u64, Texture, TextureView)>,
    sampler: Sampler,
}

impl EguiPass {
    pub fn new(device_context: &DeviceContext, pipeline_manager: &PipelineManager) -> Self {
        let root_signature = cgen::pipeline_layout::EguiPipelineLayout::root_signature();

        let mut vertex_layout = VertexLayout::default();
        vertex_layout.attributes[0] = Some(VertexLayoutAttribute {
            format: Format::R32G32_SFLOAT,
            buffer_index: 0,
            location: 0,
            byte_offset: 0,
        });
        vertex_layout.attributes[1] = Some(VertexLayoutAttribute {
            format: Format::R32G32_SFLOAT,
            buffer_index: 0,
            location: 1,
            byte_offset: 8,
        });
        vertex_layout.attributes[2] = Some(VertexLayoutAttribute {
            format: Format::R32_UINT,
            buffer_index: 0,
            location: 2,
            byte_offset: 16,
        });
        vertex_layout.buffers[0] = Some(VertexLayoutBuffer {
            stride: 20,
            rate: VertexAttributeRate::Vertex,
        });

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::egui_shader::ID,
                    cgen::shader::egui_shader::TOTO,
                ),
            )
            .unwrap();
        let pipeline_handle =
            pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader,
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_alpha_enabled(),
                depth_state: DepthState::default(),
                rasterizer_state: RasterizerState::default(),
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            }));

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
        let sampler = device_context.create_sampler(sampler_def);

        Self {
            pipeline_handle,
            texture_data: None,
            sampler,
        }
    }

    pub fn update_font_texture(
        &mut self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        egui_ctx: &egui::CtxRef,
    ) {
        if let Some((version, ..)) = self.texture_data {
            if version == egui_ctx.font_image().version {
                return;
            }
        }

        let egui_font_image = &egui_ctx.font_image();

        let texture = render_context.device_context.create_texture(
            TextureDef {
                extents: Extents3D {
                    width: egui_font_image.width as u32,
                    height: egui_font_image.height as u32,
                    depth: 1,
                },
                array_length: 1,
                mip_count: 1,
                format: Format::R8_SRGB,
                usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                memory_usage: MemoryUsage::GpuOnly,
                tiling: TextureTiling::Optimal,
            },
            "egui font",
        );

        let texture_view = texture.create_view(TextureViewDef::as_shader_resource_view(
            texture.definition(),
        ));

        let staging_buffer = render_context.device_context.create_buffer(
            BufferDef::for_staging_buffer_data(&egui_font_image.pixels, ResourceUsage::empty()),
            "staging_buffer",
        );

        staging_buffer.copy_to_host_visible_buffer(&egui_font_image.pixels);

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                &texture,
                ResourceState::UNDEFINED,
                ResourceState::COPY_DST,
            )],
        );

        cmd_buffer.cmd_copy_buffer_to_texture(
            &staging_buffer,
            &texture,
            &CmdCopyBufferToTextureParams::default(),
        );

        cmd_buffer.cmd_resource_barrier(
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
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        render_surface: &RenderSurface,
        egui: &Egui,
    ) {
        cmd_buffer.with_label("egui", |cmd_buffer| {
            cmd_buffer.cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.hdr_rt().rtv(),
                    load_op: LoadOp::Load,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue([0.0; 4]),
                }],
                &None,
            );

            let pipeline = render_context
                .pipeline_manager
                .get_pipeline(self.pipeline_handle)
                .unwrap();

            cmd_buffer.cmd_bind_pipeline(pipeline);

            let clipped_meshes = egui.tessellate();

            let mut descriptor_set = cgen::descriptor_set::EguiDescriptorSet::default();
            descriptor_set.set_font_texture(&self.texture_data.as_ref().unwrap().2);
            descriptor_set.set_font_sampler(&self.sampler);

            let descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::EguiDescriptorSet::descriptor_set_layout(),
                descriptor_set.descriptor_refs(),
            );
            cmd_buffer.cmd_bind_descriptor_set_handle(
                cgen::descriptor_set::EguiDescriptorSet::descriptor_set_layout(),
                descriptor_set_handle,
            );

            for egui::ClippedMesh(_clip_rect, mesh) in clipped_meshes {
                if mesh.is_empty() {
                    continue;
                }

                let transient_buffer = render_context
                    .transient_buffer_allocator
                    .copy_data_slice(&mesh.vertices, ResourceUsage::AS_VERTEX_BUFFER);

                cmd_buffer.cmd_bind_vertex_buffer(0, transient_buffer.vertex_buffer_binding());

                let transient_buffer = render_context
                    .transient_buffer_allocator
                    .copy_data_slice(&mesh.indices, ResourceUsage::AS_INDEX_BUFFER);

                cmd_buffer.cmd_bind_index_buffer(
                    transient_buffer.index_buffer_binding(IndexType::Uint32),
                );

                let scale = 1.0;
                let mut push_constant_data = cgen::cgen_type::EguiPushConstantData::default();
                push_constant_data.set_scale(Vec2::new(scale, scale).into());
                push_constant_data.set_translation(Vec2::new(0.0, 0.0).into());
                push_constant_data.set_width(
                    (render_surface.extents().width() as f32 / egui.ctx().pixels_per_point())
                        .into(),
                );
                push_constant_data.set_height(
                    (render_surface.extents().height() as f32 / egui.ctx().pixels_per_point())
                        .into(),
                );

                cmd_buffer.cmd_push_constant_typed(&push_constant_data);
                cmd_buffer.cmd_draw_indexed(mesh.indices.len() as u32, 0, 0);
            }
            cmd_buffer.cmd_end_render_pass();
        });
    }
}
