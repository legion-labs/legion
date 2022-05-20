use crate::{
    core::render_graph::RenderGraphBuilder,
    core::render_graph::{
        AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
        PostProcessPass, SSAOPass, UiPass,
    },
    core::render_graph::{
        RenderGraph, RenderGraphResourceDef, RenderGraphResourceId, RenderGraphTextureDef,
        RenderGraphTextureViewDef, RenderGraphViewDef, RenderGraphViewId,
    },
    resources::PipelineManager,
};

use lgn_graphics_api::{
    DeviceContext, Format, GPUViewType, GfxError, GfxResult, PlaneSlice, Texture, ViewDimension,
};

use super::RenderGraphLoadState;

///
///
/// `https://logins.github.io/graphics/2021/05/31/RenderGraphs.html`
/// `https://medium.com/embarkstudios/homegrown-rendering-with-rust-1e39068e56a7`
///
///
///
///

pub struct RenderView {
    pub target: Texture,
}

pub struct Config {
    display_post_process: bool,
    display_ui: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display_post_process: true,
            display_ui: true,
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

pub struct RenderScript {
    // Passes
    pub gpu_culling_pass: GpuCullingPass,
    pub depth_layer_pass: DepthLayerPass,
    pub opaque_layer_pass: OpaqueLayerPass,
    pub ssao_pass: SSAOPass,
    pub alphablended_layer_pass: AlphaBlendedLayerPass,
    pub postprocess_pass: PostProcessPass,
    pub lighting_pass: LightingPass,
    pub ui_pass: UiPass,

    // Resources
    pub prev_depth: Texture,
}

impl RenderScript {
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
        view: &RenderView,
        config: &Config,
        pipeline_manager: &PipelineManager,
        device_context: &DeviceContext,
    ) -> GfxResult<RenderGraph> {
        let mut render_graph_builder = RenderGraph::builder(pipeline_manager, device_context);

        if view.target.definition().extents.width == 0
            || view.target.definition().extents.height == 0
            || view.target.definition().extents.depth != 1
            || view.target.definition().array_length != 1
        {
            return Err(GfxError::String("View target is invalid".to_string()));
        }

        let view_target_id = render_graph_builder.inject_render_target("ViewTarget", &view.target);

        let depth_buffer_desc = self.make_depth_buffer_desc(view);
        let depth_buffer_id =
            render_graph_builder.declare_render_target("DepthBuffer", &depth_buffer_desc);
        let depth_view_def = self.make_single_mip_depth_view_def(depth_buffer_id);
        let depth_view_id = render_graph_builder.declare_view(&depth_view_def);
        let depth_read_only_view_def =
            self.make_single_mip_depth_read_only_view_def(depth_buffer_id);
        let depth_read_only_view_id = render_graph_builder.declare_view(&depth_read_only_view_def);

        let gbuffer_descs = self.make_gbuffer_descs(view);
        let gbuffer_ids = [
            render_graph_builder.declare_render_target("GBuffer0", &gbuffer_descs[0]),
            render_graph_builder.declare_render_target("GBuffer1", &gbuffer_descs[1]),
            render_graph_builder.declare_render_target("GBuffer2", &gbuffer_descs[2]),
            render_graph_builder.declare_render_target("GBuffer3", &gbuffer_descs[3]),
        ];
        let gbuffer_read_view_ids = [
            render_graph_builder.declare_view(&self.make_single_mip_color_view_def(gbuffer_ids[0])),
            render_graph_builder.declare_view(&self.make_single_mip_color_view_def(gbuffer_ids[1])),
            render_graph_builder.declare_view(&self.make_single_mip_color_view_def(gbuffer_ids[2])),
            render_graph_builder.declare_view(&self.make_single_mip_color_view_def(gbuffer_ids[3])),
        ];
        let gbuffer_write_view_ids = [
            render_graph_builder
                .declare_view(&self.make_single_mip_color_write_view_def(gbuffer_ids[0])),
            render_graph_builder
                .declare_view(&self.make_single_mip_color_write_view_def(gbuffer_ids[1])),
            render_graph_builder
                .declare_view(&self.make_single_mip_color_write_view_def(gbuffer_ids[2])),
            render_graph_builder
                .declare_view(&self.make_single_mip_color_write_view_def(gbuffer_ids[3])),
        ];

        let radiance_buffer_desc = self.make_radiance_buffer_desc(view);
        let radiance_buffer_id =
            render_graph_builder.declare_render_target("RadianceBuffer", &radiance_buffer_desc);
        let radiance_write_uav_view_id = render_graph_builder
            .declare_view(&self.make_single_mip_color_write_uav_view_def(radiance_buffer_id));
        let radiance_write_rt_view_id = render_graph_builder
            .declare_view(&self.make_single_mip_color_write_view_def(radiance_buffer_id));
        let radiance_read_view_id = render_graph_builder
            .declare_view(&self.make_single_mip_color_view_def(radiance_buffer_id));

        let ao_buffer_desc = self.make_ao_buffer_desc(view);
        let ao_buffer_id = render_graph_builder.declare_render_target("AOBuffer", &ao_buffer_desc);
        let ao_write_view_id = render_graph_builder
            .declare_view(&self.make_single_mip_color_write_view_def(ao_buffer_id));
        let ao_read_view_id =
            render_graph_builder.declare_view(&self.make_single_mip_color_view_def(ao_buffer_id));

        render_graph_builder = self
            .depth_layer_pass
            .build_render_graph(render_graph_builder, depth_view_id);
        render_graph_builder = self.gpu_culling_pass.build_render_graph(
            render_graph_builder,
            depth_buffer_id,
            depth_read_only_view_id,
        );
        render_graph_builder = self.opaque_layer_pass.build_render_graph(
            render_graph_builder,
            depth_read_only_view_id,
            gbuffer_write_view_ids,
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

        if config.display_post_process() {
            render_graph_builder = self
                .postprocess_pass
                .build_render_graph(render_graph_builder, radiance_read_view_id);
        }

        let ui_buffer_and_view_ids = if config.display_ui() {
            let ui_buffer_desc = self.make_ui_buffer_desc(view);
            let ui_buffer_id =
                render_graph_builder.declare_render_target("UIBuffer", &ui_buffer_desc);
            let ui_write_view_id = render_graph_builder
                .declare_view(&self.make_single_mip_color_write_view_def(ui_buffer_id));
            let ui_read_view_id = render_graph_builder
                .declare_view(&self.make_single_mip_color_view_def(ui_buffer_id));
            render_graph_builder = self
                .ui_pass
                .build_render_graph(render_graph_builder, ui_write_view_id);
            Some((ui_buffer_id, ui_read_view_id))
        } else {
            None
        };

        let view_target_view_id = render_graph_builder
            .declare_view(&self.make_single_mip_color_write_view_def(view_target_id));

        render_graph_builder = self.combine_pass(
            render_graph_builder,
            view_target_view_id,
            radiance_read_view_id,
            ui_buffer_and_view_ids,
        );

        Ok(render_graph_builder.build())
    }

    #[allow(clippy::unused_self)]
    fn make_depth_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::D32_SFLOAT,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::ShaderResource,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_write_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::RenderTarget,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_write_uav_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::UnorderedAccess,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_depth_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::DepthStencil,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Depth,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_depth_read_only_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::DepthStencil,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Depth,
            first_array_slice: 0,
            array_size: 1,
            read_only: true,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_gbuffer_descs(&self, view: &RenderView) -> Vec<RenderGraphResourceDef> {
        let mut texture_def = RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::R16G16B16A16_SFLOAT,
        };

        // Just to simulate different formats
        let gbuffer0_def = texture_def.clone();
        texture_def.format = Format::R16G16_SNORM; // Normals
        let gbuffer1_def = texture_def.clone();
        texture_def.format = Format::R8G8B8A8_UNORM;
        let gbuffer2_def = texture_def.clone();
        texture_def.format = Format::R8G8B8A8_UNORM;
        let gbuffer3_def = texture_def;

        vec![
            RenderGraphResourceDef::Texture(gbuffer0_def),
            RenderGraphResourceDef::Texture(gbuffer1_def),
            RenderGraphResourceDef::Texture(gbuffer2_def),
            RenderGraphResourceDef::Texture(gbuffer3_def),
        ]
    }

    #[allow(clippy::unused_self)]
    fn make_ao_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_radiance_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::R16G16B16A16_SFLOAT,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_ui_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
        })
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_arguments)]
    fn combine_pass<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
        ui_buffer_and_view_ids: Option<(RenderGraphResourceId, RenderGraphViewId)>,
    ) -> RenderGraphBuilder<'a> {
        builder.add_compute_pass("Combine", |mut compute_pass_builder| {
            compute_pass_builder = compute_pass_builder
                .read(radiance_view_id, RenderGraphLoadState::Load)
                .write(view_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _| {
                    println!("Combine pass execute");
                });
            if let Some(ui_buffer_and_view_ids) = ui_buffer_and_view_ids {
                compute_pass_builder =
                    compute_pass_builder.read(ui_buffer_and_view_ids.1, RenderGraphLoadState::Load);
            }
            compute_pass_builder
        })
    }
}
