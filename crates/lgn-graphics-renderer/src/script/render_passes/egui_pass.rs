use lgn_graphics_api::{
    AddressMode, BlendState, BufferDef, CmdCopyBufferToTextureParams, CompareOp, DepthState,
    Extents3D, FilterType, Format, GraphicsPipelineDef, IndexType, MemoryUsage, MipMapMode,
    PrimitiveTopology, RasterizerState, ResourceFlags, ResourceState, ResourceUsage, SampleCount,
    SamplerDef, TextureBarrier, TextureDef, TextureTiling, TextureViewDef, VertexAttributeRate,
    VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_math::Vec2;

use crate::{
    cgen,
    core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId},
    resources::PipelineDef,
    script::RenderView,
};

pub struct EguiPass;

impl EguiPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view: &RenderView<'_>,
        radiance_write_rt_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
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

        let shader = builder
            .pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::egui_shader::ID,
                    cgen::shader::egui_shader::TOTO,
                ),
            )
            .unwrap();
        let pipeline_handle = builder
            .pipeline_manager
            .register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
                shader,
                root_signature: root_signature.clone(),
                vertex_layout,
                blend_state: BlendState::default_premultiplied_alpha(),
                depth_state: DepthState::default(),
                rasterizer_state: RasterizerState::default(),
                color_formats: vec![Format::R16G16B16A16_SFLOAT],
                sample_count: SampleCount::SampleCount1,
                depth_stencil_format: None,
                primitive_topology: PrimitiveTopology::TriangleList,
            }));

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
        let sampler = builder.device_context.create_sampler(sampler_def);

        let view_target_extents = *view.target.extents();

        builder.add_scope("Egui", |builder| {
            builder
                .add_compute_pass("UpdateEguiTexture", |compute_pass_builder| {
                    compute_pass_builder.execute(|_, execute_context, cmd_buffer| {

                        // TODO(jsg): This pass needs cleanup. The font texture should be injected
                        // into the graph, and its state should be managed by the graph.

                        let egui = execute_context.debug_stuff.egui;

                        if egui.is_enabled() {
                            let egui_pass =
                                execute_context.debug_stuff.render_surface.egui_renderpass();
                            let mut egui_pass = egui_pass.write();

                            let textures_delta = egui.textures_delta();
                            let mut textures_delta = textures_delta.lock().unwrap();
                            for (_texture_id, image_delta) in textures_delta.set.drain() {
                                match &image_delta.image {
                                    egui::epaint::ImageData::Color(_) => {
                                        // TODO(jsg): implement uploading arbitrary textures for use by egui.
                                    }
                                    egui::epaint::ImageData::Font(font_texture) => {
                                        if image_delta.is_whole() {
                                            let texture = execute_context
                                            .render_context
                                            .device_context
                                            .create_texture(
                                                TextureDef {
                                                    extents: Extents3D {
                                                        width: font_texture.size[0] as u32,
                                                        height: font_texture.size[1] as u32,
                                                        depth: 1,
                                                    },
                                                    array_length: 1,
                                                    mip_count: 1,
                                                    format: Format::R8_SRGB,
                                                    usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                                                        | ResourceUsage::AS_TRANSFERABLE,
                                                    resource_flags: ResourceFlags::empty(),
                                                    memory_usage: MemoryUsage::GpuOnly,
                                                    tiling: TextureTiling::Optimal,
                                                },
                                                "egui font",
                                            );

                                        let texture_view = texture.create_view(
                                            TextureViewDef::as_shader_resource_view(texture.definition()),
                                        );

                                        fn fast_round(r: f32) -> u8 {
                                            (r + 0.5).floor() as _ // rust does a saturating cast since 1.45
                                        }

                                        let gamma = 1.0 / 2.2;
                                        // See egui::epaint::FontImage::srgba_pixels() -- it returns R8G8B8A8 but that's a waste, R8 is enough.
                                        let data: Vec<u8> = font_texture.pixels.iter().map(move |coverage| {
                                            fast_round(coverage.powf(gamma / 2.2) * 255.0) as u8
                                        }).collect();

                                        let staging_buffer =
                                            execute_context.render_context.device_context.create_buffer(
                                                BufferDef::for_staging_buffer_data(
                                                    &data,
                                                    ResourceUsage::empty(),
                                                ),
                                                "staging_buffer",
                                            );

                                        staging_buffer.copy_to_host_visible_buffer(&data);

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

                                        egui_pass.font_texture =
                                            Some((texture, texture_view));
                                        } else {
                                            // TODO(jsg): Implement partial texture updates.
                                        }
                                    }
                                }
                            }
                        }
                    })
                })
                .add_graphics_pass("Egui draw", |graphics_pass_builder| {
                    graphics_pass_builder
                        .render_target(0, radiance_write_rt_view_id, RenderGraphLoadState::Load)
                        .execute(move |_, execute_context, cmd_buffer| {
                            let egui = execute_context.debug_stuff.egui;

                            let egui_pass =
                                execute_context.debug_stuff.render_surface.egui_renderpass();
                            let egui_pass = egui_pass.write();

                            if egui.is_enabled() && egui_pass.font_texture.is_some() {

                                if let Some(pipeline) = execute_context
                                    .render_context
                                    .pipeline_manager
                                    .get_pipeline(pipeline_handle)
                                {
                                    cmd_buffer.cmd_bind_pipeline(pipeline);

                                    let clipped_meshes = egui.tessellate();

                                    let mut descriptor_set =
                                        cgen::descriptor_set::EguiDescriptorSet::default();
                                    descriptor_set
                                        .set_font_texture(&egui_pass.font_texture.as_ref().unwrap().1);
                                    descriptor_set.set_font_sampler(&sampler);

                                    let descriptor_set_handle = execute_context
                                        .render_context
                                        .write_descriptor_set(
                                        cgen::descriptor_set::EguiDescriptorSet::descriptor_set_layout(
                                        ),
                                        descriptor_set.descriptor_refs(),
                                    );
                                    cmd_buffer.cmd_bind_descriptor_set_handle(
                                        cgen::descriptor_set::EguiDescriptorSet::descriptor_set_layout(
                                        ),
                                        descriptor_set_handle,
                                    );

                                    for egui::ClippedPrimitive {clip_rect: _clip_rect, primitive } in clipped_meshes {
                                        match &primitive {
                                            egui::epaint::Primitive::Mesh(mesh) => {
                                                if mesh.is_empty() {
                                                    continue;
                                                }

                                                let transient_buffer = execute_context
                                                    .render_context
                                                    .transient_buffer_allocator
                                                    .copy_data_slice(
                                                        &mesh.vertices,
                                                        ResourceUsage::AS_VERTEX_BUFFER,
                                                    );

                                                cmd_buffer.cmd_bind_vertex_buffer(
                                                    0,
                                                    transient_buffer.vertex_buffer_binding(),
                                                );

                                                let transient_buffer = execute_context
                                                    .render_context
                                                    .transient_buffer_allocator
                                                    .copy_data_slice(
                                                        &mesh.indices,
                                                        ResourceUsage::AS_INDEX_BUFFER,
                                                    );

                                                cmd_buffer.cmd_bind_index_buffer(
                                                    transient_buffer
                                                        .index_buffer_binding(IndexType::Uint32),
                                                );

                                                let scale = 1.0;
                                                let mut push_constant_data =
                                                    cgen::cgen_type::EguiPushConstantData::default();
                                                push_constant_data
                                                    .set_scale(Vec2::new(scale, scale).into());
                                                push_constant_data
                                                    .set_translation(Vec2::new(0.0, 0.0).into());
                                                push_constant_data.set_width(
                                                    (view_target_extents.width as f32
                                                        / egui.context().pixels_per_point())
                                                    .into(),
                                                );
                                                push_constant_data.set_height(
                                                    (view_target_extents.height as f32
                                                        / egui.context().pixels_per_point())
                                                    .into(),
                                                );

                                                cmd_buffer.cmd_push_constant_typed(&push_constant_data);
                                                cmd_buffer.cmd_draw_indexed(
                                                    mesh.indices.len() as u32,
                                                    0,
                                                    0,
                                                );
                                            }
                                            egui::epaint::Primitive::Callback(_) => {
                                                // TODO(jsg): Implement callback primitive type.
                                            }
                                        }
                                    }
                                }
                            }
                        })
                })
        })
    }
}
