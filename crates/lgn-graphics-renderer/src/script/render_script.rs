use crate::{
    cgen,
    core::RenderGraphBuilder,
    core::RenderGraphLoadState,
    core::{
        BinaryWriter, GpuUploadManager, RenderGraph, RenderGraphResourceId, RenderGraphViewId,
        RenderResources, UploadGPUBuffer, UploadGPUResource,
    },
    gpu_renderer::{DefaultLayers, MeshRenderer},
    resources::{PipelineDef, PipelineHandle, PipelineManager, UnifiedStaticBuffer},
    RenderContext,
};

use lgn_graphics_api::{
    AddressMode, BlendState, CompareOp, CullMode, DepthState, DeviceContext, FilterType, Format,
    GfxError, GfxResult, GraphicsPipelineDef, MipMapMode, PrimitiveTopology, RasterizerState,
    ResourceState, SampleCount, SamplerDef, StencilOp, Texture, VertexLayout,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_tracing::span_scope;

use super::render_passes::{
    AlphaBlendedLayerPass, DebugPass, EguiPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PickingPass, PostProcessPass, SSAOPass, UiPass,
};

///
///
/// `https://logins.github.io/graphics/2021/05/31/RenderGraphs.html`
/// `https://medium.com/embarkstudios/homegrown-rendering-with-rust-1e39068e56a7`
/// `https://blog.traverseresearch.nl/render-graph-101-f42646255636`
///
///
///

pub struct RenderView<'a> {
    pub target: &'a Texture,
}

pub struct Config {
    pub display_post_process: bool,
    pub display_ui: bool,
    pub frame_idx: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display_post_process: true,
            display_ui: true,
            frame_idx: 0,
        }
    }
}

impl Config {
    pub fn display_post_process(&self) -> bool {
        self.display_post_process
    }

    pub fn display_ui(&self) -> bool {
        self.display_ui
    }
}

pub struct RenderScript<'a> {
    // Passes
    pub gpu_culling_pass: GpuCullingPass,
    pub picking_pass: PickingPass,
    pub opaque_layer_pass: OpaqueLayerPass,
    pub ssao_pass: SSAOPass,
    pub alphablended_layer_pass: AlphaBlendedLayerPass,
    pub debug_pass: DebugPass,
    pub postprocess_pass: PostProcessPass,
    pub lighting_pass: LightingPass,
    pub ui_pass: UiPass,
    pub egui_pass: EguiPass,

    // Resources
    pub hzb: [&'a Texture; 2],
}

impl RenderScript<'_> {
    // IMPORTANT:
    // The list of passes that are created and called by RenderScript is temporary. Some
    // things are there just to reproduce the old rendering path, and others are there
    // in preparation of future render passes (but do nothing for now).

    /// .
    ///
    /// # Examples
    ///
    /// ```
    /// use lgn_graphics_renderer::core::render_script::RenderScript;
    ///
    /// let mut render_script = ...;
    /// let result = render_script.build_render_graph(view, config);
    /// assert_eq!(result, );
    /// assert_eq!(render_script, );
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub(crate) fn build_render_graph(
        &mut self,
        view: &RenderView<'_>,
        config: &Config,
        render_resources: &RenderResources,
        pipeline_manager: &mut PipelineManager,
        device_context: &DeviceContext,
        hzb_cleared: bool,
    ) -> GfxResult<RenderGraph> {
        span_scope!("build_render_graph");

        let mut render_graph_builder =
            RenderGraph::builder(render_resources, pipeline_manager, device_context);

        if view.target.definition().extents.width == 0
            || view.target.definition().extents.height == 0
            || view.target.definition().extents.depth != 1
            || view.target.definition().array_length != 1
        {
            return Err(GfxError::String("View target is invalid".to_string()));
        }

        let w = view.target.extents().width;
        let h = view.target.extents().height;

        //----------------------------------------------------------------
        // Inject external resources

        // TODO(jsg): need to think of a better way of doing this.

        let prev_hzb_idx = config.frame_idx as usize % 2;
        let current_hzb_idx = (config.frame_idx + 1) as usize % 2;

        let mut names = vec!["PrevFrameHZB", "CurrentFrameHZB"];
        if prev_hzb_idx == 0 {
            names = names.into_iter().rev().collect();
        }

        // Initial state is important because we need the contents of the previous_frame_hzb.
        let hzb_ids = [
            render_graph_builder.inject_render_target(
                names[0],
                self.hzb[0],
                if hzb_cleared {
                    ResourceState::RENDER_TARGET
                } else {
                    ResourceState::SHADER_RESOURCE
                },
            ),
            render_graph_builder.inject_render_target(
                names[1],
                self.hzb[1],
                if hzb_cleared {
                    ResourceState::RENDER_TARGET
                } else {
                    ResourceState::SHADER_RESOURCE
                },
            ),
        ];

        let prev_hzb_id = hzb_ids[prev_hzb_idx];
        let current_hzb_id = hzb_ids[current_hzb_idx];

        let prev_hzb_srv_id = render_graph_builder.declare_texture_srv_with_mips(
            prev_hzb_id,
            0,
            self.hzb[prev_hzb_idx].definition().mip_count,
        );

        // Initial state doesn't matter because we don't use the previous contents.
        let view_target_id = render_graph_builder.inject_render_target(
            "ViewTarget",
            view.target,
            ResourceState::UNDEFINED,
        );

        //----------------------------------------------------------------
        // Declare resources and views

        let depth_buffer_id =
            render_graph_builder.declare_render_target("DepthBuffer", w, h, Format::D32_SFLOAT);
        let depth_view_id = render_graph_builder.declare_depth_texture_dsv(depth_buffer_id, false);
        let depth_read_only_view_id =
            render_graph_builder.declare_depth_texture_dsv(depth_buffer_id, true);

        let gbuffer_ids = [
            render_graph_builder.declare_render_target(
                "GBuffer0",
                w,
                h,
                Format::R16G16B16A16_SFLOAT,
            ),
            render_graph_builder.declare_render_target("GBuffer1", w, h, Format::R16G16_SNORM),
            render_graph_builder.declare_render_target("GBuffer2", w, h, Format::R8G8B8A8_UNORM),
            render_graph_builder.declare_render_target("GBuffer3", w, h, Format::R8G8B8A8_UNORM),
        ];
        let gbuffer_read_view_ids = [
            render_graph_builder.declare_texture_srv(gbuffer_ids[0]),
            render_graph_builder.declare_texture_srv(gbuffer_ids[1]),
            render_graph_builder.declare_texture_srv(gbuffer_ids[2]),
            render_graph_builder.declare_texture_srv(gbuffer_ids[3]),
        ];
        let gbuffer_write_view_ids = [
            render_graph_builder.declare_texture_rtv(gbuffer_ids[0]),
            render_graph_builder.declare_texture_rtv(gbuffer_ids[1]),
            render_graph_builder.declare_texture_rtv(gbuffer_ids[2]),
            render_graph_builder.declare_texture_rtv(gbuffer_ids[3]),
        ];

        let radiance_buffer_id = render_graph_builder.declare_render_target(
            "RadianceBuffer",
            w,
            h,
            Format::R16G16B16A16_SFLOAT,
        );
        let radiance_write_uav_view_id =
            render_graph_builder.declare_texture_uav(radiance_buffer_id);
        let radiance_write_rt_view_id =
            render_graph_builder.declare_texture_rtv(radiance_buffer_id);
        let radiance_read_view_id = render_graph_builder.declare_texture_srv(radiance_buffer_id);

        let ao_buffer_id =
            render_graph_builder.declare_render_target("AOBuffer", w, h, Format::R8_UNORM);
        let ao_write_view_id = render_graph_builder.declare_texture_uav(ao_buffer_id);
        let ao_read_view_id = render_graph_builder.declare_texture_srv(ao_buffer_id);

        //----------------------------------------------------------------
        // Build graph
        //
        // TODO(jsg): Passes still missing:
        //       * egui
        //       * picking

        let mut count_buffer_size: u64 = 0;
        let mut indirect_arg_buffer_size: u64 = 0;
        let mut depth_count_buffer_size: u64 = 0;

        {
            let mut mesh_renderer = render_resources.get_mut::<MeshRenderer>();

            for (index, layer) in mesh_renderer.default_layers.iter_mut().enumerate() {
                let per_state_offsets =
                    layer.aggregate_offsets(&mut count_buffer_size, &mut indirect_arg_buffer_size);
                if index == DefaultLayers::Depth as usize {
                    depth_count_buffer_size = count_buffer_size;
                }

                if !per_state_offsets.is_empty() {
                    let mut binary_writer = BinaryWriter::new();
                    binary_writer.write_slice(&per_state_offsets);

                    // Update buffer at layer.state_page.byte_offset()
                    let mut upload_manager = render_resources.get_mut::<GpuUploadManager>();
                    let unified_static_buffer = render_resources.get::<UnifiedStaticBuffer>();
                    upload_manager.push(UploadGPUResource::Buffer(UploadGPUBuffer {
                        src_data: binary_writer.take(),
                        dst_buffer: unified_static_buffer.buffer().clone(),
                        dst_offset: layer.state_page.byte_offset(),
                    }));
                }
            }
        }

        let draw_count_buffer_id = render_graph_builder.declare_buffer(
            "DrawCountBuffer",
            std::mem::size_of::<u32>() as u64,
            count_buffer_size.max(1),
        );

        let draw_args_buffer_id = render_graph_builder.declare_buffer(
            "DrawArgsBuffer",
            5 * std::mem::size_of::<u32>() as u64,
            indirect_arg_buffer_size.max(1),
        );

        render_graph_builder = self.gpu_culling_pass.build_render_graph(
            render_graph_builder,
            depth_buffer_id,
            depth_view_id,
            draw_count_buffer_id,
            draw_args_buffer_id,
            depth_count_buffer_size,
            self.hzb[prev_hzb_idx].definition(),
            prev_hzb_srv_id,
            self.hzb[current_hzb_idx].definition(),
            current_hzb_id,
        );
        render_graph_builder = self.picking_pass.build_render_graph(
            render_graph_builder,
            view,
            gbuffer_write_view_ids[0],
            draw_count_buffer_id,
            draw_args_buffer_id,
        );
        render_graph_builder = self.opaque_layer_pass.build_render_graph(
            render_graph_builder,
            depth_read_only_view_id,
            gbuffer_write_view_ids,
            draw_count_buffer_id,
            draw_args_buffer_id,
        );
        render_graph_builder = self.ssao_pass.build_render_graph(
            render_graph_builder,
            view,
            depth_read_only_view_id,
            gbuffer_read_view_ids,
            ao_write_view_id,
        );
        render_graph_builder = self.lighting_pass.build_render_graph(
            render_graph_builder,
            depth_read_only_view_id,
            gbuffer_read_view_ids,
            ao_read_view_id,
            radiance_write_uav_view_id,
        );
        render_graph_builder = self.alphablended_layer_pass.build_render_graph(
            render_graph_builder,
            depth_read_only_view_id,
            radiance_write_rt_view_id,
        );
        render_graph_builder = self.debug_pass.build_render_graph(
            render_graph_builder,
            view,
            depth_view_id,
            gbuffer_write_view_ids[0], // radiance_write_rt_view_id
        );
        render_graph_builder = self.egui_pass.build_render_graph(
            render_graph_builder,
            view,
            gbuffer_write_view_ids[0], // radiance_write_rt_view_id
        );

        if config.display_post_process() {
            render_graph_builder = self
                .postprocess_pass
                .build_render_graph(render_graph_builder, radiance_read_view_id);
        }

        let ui_buffer_and_view_ids = if config.display_ui() {
            let ui_buffer_id = render_graph_builder.declare_render_target(
                "UIBuffer",
                w,
                h,
                Format::R8G8B8A8_UNORM,
            );
            let ui_write_view_id = render_graph_builder.declare_texture_rtv(ui_buffer_id);
            let ui_read_view_id = render_graph_builder.declare_texture_srv(ui_buffer_id);
            render_graph_builder = self
                .ui_pass
                .build_render_graph(render_graph_builder, ui_write_view_id);
            Some((ui_buffer_id, ui_read_view_id))
        } else {
            None
        };

        let view_target_view_id = render_graph_builder.declare_texture_rtv(view_target_id);

        render_graph_builder = self.combine_pass(
            render_graph_builder,
            view_target_view_id,
            gbuffer_read_view_ids[0], //radiance_read_view_id,
            ui_buffer_and_view_ids,
        );

        Ok(render_graph_builder.build())
    }

    fn build_final_resolve_pso(pipeline_manager: &mut PipelineManager) -> PipelineHandle {
        let root_signature = cgen::pipeline_layout::FinalResolvePipelineLayout::root_signature();

        let depth_state = DepthState {
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: CompareOp::Never,
            stencil_test_enable: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            front_depth_fail_op: StencilOp::default(),
            front_stencil_compare_op: CompareOp::default(),
            front_stencil_fail_op: StencilOp::default(),
            front_stencil_pass_op: StencilOp::default(),
            back_depth_fail_op: StencilOp::default(),
            back_stencil_compare_op: CompareOp::default(),
            back_stencil_fail_op: StencilOp::default(),
            back_stencil_pass_op: StencilOp::default(),
        };

        let rasterizer_state = lgn_graphics_api::RasterizerState {
            cull_mode: CullMode::Back,
            ..RasterizerState::default()
        };

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::final_resolve_shader::ID,
                    cgen::shader::final_resolve_shader::NONE,
                ),
            )
            .unwrap();
        pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
            shader,
            root_signature: root_signature.clone(),
            vertex_layout: VertexLayout::default(),
            blend_state: BlendState::default_alpha_disabled(),
            depth_state,
            rasterizer_state,
            color_formats: vec![Format::B8G8R8A8_UNORM],
            sample_count: SampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: PrimitiveTopology::TriangleList,
        }))
    }

    #[allow(clippy::unused_self)]
    fn combine_pass<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
        ui_buffer_and_view_ids: Option<(RenderGraphResourceId, RenderGraphViewId)>,
    ) -> RenderGraphBuilder<'a> {
        let pipeline_handle = Self::build_final_resolve_pso(builder.pipeline_manager);

        let linear_sampler = builder.device_context.create_sampler(SamplerDef {
            min_filter: FilterType::Nearest,
            mag_filter: FilterType::Nearest,
            mip_map_mode: MipMapMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            compare_op: CompareOp::Never,
        });

        builder.add_graphics_pass("Combine", |mut graphics_pass_builder| {
            if let Some(ui_buffer_and_view_ids) = ui_buffer_and_view_ids {
                graphics_pass_builder = graphics_pass_builder
                    .read(ui_buffer_and_view_ids.1, RenderGraphLoadState::Load);
            }
            graphics_pass_builder
                .read(radiance_view_id, RenderGraphLoadState::Load)
                .render_target(0, view_view_id, RenderGraphLoadState::DontCare)
                .execute(move |context, execute_context, cmd_buffer| {
                    let render_context: &mut RenderContext<'_> = execute_context.render_context;

                    if let Some(pipeline) = render_context
                        .pipeline_manager
                        .get_pipeline(pipeline_handle)
                    {
                        cmd_buffer.cmd_bind_pipeline(pipeline);

                        let mut descriptor_set =
                            cgen::descriptor_set::FinalResolveDescriptorSet::default();
                        descriptor_set
                            .set_linear_texture(context.get_texture_view(radiance_view_id));
                        descriptor_set.set_linear_sampler(&linear_sampler);

                        let descriptor_set_handle = render_context.write_descriptor_set(
                            cgen::descriptor_set::FinalResolveDescriptorSet::descriptor_set_layout(
                            ),
                            descriptor_set.descriptor_refs(),
                        );
                        cmd_buffer.cmd_bind_descriptor_set_handle(
                            cgen::descriptor_set::FinalResolveDescriptorSet::descriptor_set_layout(
                            ),
                            descriptor_set_handle,
                        );

                        cmd_buffer.cmd_draw(3, 0);
                    }
                })
        })
    }
}
