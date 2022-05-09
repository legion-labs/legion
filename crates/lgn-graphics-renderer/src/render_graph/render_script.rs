use crate::{
    render_graph::RenderGraphBuilder,
    render_graph::{
        AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
        PostProcessPass, SSAOPass, UiPass,
    },
    render_graph::{
        RenderGraph, RenderGraphResourceDef, RenderGraphResourceId, RenderGraphTextureDef,
        RenderGraphTextureViewDef, RenderGraphViewDef, RenderGraphViewId,
    },
};

use lgn_graphics_api::{Format, GfxError, GfxResult, PlaneSlice, Texture, ViewDimension};

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
    /// use lgn_graphics_tests::render_script::RenderScript;
    ///
    /// let mut render_script = ;
    /// let result = render_script.build_render_graph(view, config);
    /// assert_eq!(result, );
    /// assert_eq!(render_script, );
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn build_render_graph(
        &mut self,
        view: &RenderView,
        config: &Config,
    ) -> GfxResult<RenderGraph> {
        let mut rendergraph_builder = RenderGraph::builder();

        if view.target.definition().extents.width == 0
            || view.target.definition().extents.height == 0
            || view.target.definition().extents.depth != 1
            || view.target.definition().array_length != 1
        {
            return Err(GfxError::String("View target is invalid".to_string()));
        }

        let view_target_id = rendergraph_builder.inject_render_target("ViewTarget", &view.target);

        let depth_buffer_desc = self.make_depth_buffer_desc(view);
        let depth_buffer_id =
            rendergraph_builder.declare_render_target("DepthBuffer", &depth_buffer_desc);
        let single_mip_color_view_def = self.make_single_mip_color_view_def();
        let single_mip_color_view_id = rendergraph_builder.declare_view(&single_mip_color_view_def);
        let single_mip_depth_view_def = self.make_single_mip_depth_view_def();
        let single_mip_depth_view_id = rendergraph_builder.declare_view(&single_mip_depth_view_def);
        let single_mip_depth_read_only_view_def = self.make_single_mip_depth_read_only_view_def();
        let single_mip_depth_read_only_view_id =
            rendergraph_builder.declare_view(&single_mip_depth_read_only_view_def);

        let gbuffer_descs = self.make_gbuffer_descs(view);
        let gbuffer_ids = [
            rendergraph_builder.declare_render_target("GBuffer0", &gbuffer_descs[0]),
            rendergraph_builder.declare_render_target("GBuffer1", &gbuffer_descs[1]),
            rendergraph_builder.declare_render_target("GBuffer2", &gbuffer_descs[2]),
            rendergraph_builder.declare_render_target("GBuffer3", &gbuffer_descs[3]),
        ];

        let radiance_buffer_desc = self.make_radiance_buffer_desc(view);
        let radiance_buffer_id =
            rendergraph_builder.declare_render_target("RadianceBuffer", &radiance_buffer_desc);

        let ao_buffer_desc = self.make_ao_buffer_desc(view);
        let ao_buffer_id = rendergraph_builder.declare_render_target("AOBuffer", &ao_buffer_desc);

        rendergraph_builder = self.depth_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_depth_view_id,
        );
        rendergraph_builder = self.gpu_culling_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_depth_read_only_view_id,
        );
        rendergraph_builder = self.opaque_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_depth_read_only_view_id,
            gbuffer_ids,
            single_mip_color_view_id,
        );
        rendergraph_builder = self.ssao_pass.build_render_graph(
            rendergraph_builder,
            view,
            depth_buffer_id,
            single_mip_depth_read_only_view_id,
            gbuffer_ids,
            single_mip_color_view_id,
            ao_buffer_id,
            single_mip_color_view_id,
        );
        rendergraph_builder = self.lighting_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_depth_read_only_view_id,
            gbuffer_ids,
            single_mip_color_view_id,
            ao_buffer_id,
            single_mip_color_view_id,
            radiance_buffer_id,
            single_mip_color_view_id,
        );
        rendergraph_builder = self.alphablended_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_depth_read_only_view_id,
            radiance_buffer_id,
            single_mip_color_view_id,
        );

        if config.display_post_process() {
            rendergraph_builder = self.postprocess_pass.build_render_graph(
                rendergraph_builder,
                radiance_buffer_id,
                single_mip_color_view_id,
            );
        }

        let ui_buffer_id = if config.display_ui() {
            let ui_buffer_desc = self.make_ui_buffer_desc(view);
            let ui_buffer_id =
                rendergraph_builder.declare_render_target("UIBuffer", &ui_buffer_desc);
            rendergraph_builder = self.ui_pass.build_render_graph(
                rendergraph_builder,
                ui_buffer_id,
                single_mip_color_view_id,
            );
            Some(ui_buffer_id)
        } else {
            None
        };

        rendergraph_builder = self.combine_pass(
            rendergraph_builder,
            view_target_id,
            single_mip_color_view_id,
            radiance_buffer_id,
            single_mip_color_view_id,
            ui_buffer_id,
            single_mip_color_view_id,
        );

        Ok(rendergraph_builder.build())
    }

    #[allow(clippy::unused_self)]
    fn make_depth_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::D24_UNORM_S8_UINT,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_view_def(&self) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
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
    fn make_single_mip_depth_view_def(&self) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
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
    fn make_single_mip_depth_read_only_view_def(&self) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
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
            format: Format::R8G8B8A8_UNORM,
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
    fn combine_pass(
        &self,
        builder: RenderGraphBuilder,
        view_target_id: RenderGraphResourceId,
        view_view_id: RenderGraphViewId,
        radiance_buffer_id: RenderGraphResourceId,
        radiance_view_id: RenderGraphViewId,
        ui_buffer_id: Option<RenderGraphResourceId>,
        ui_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_compute_pass("Combine", |compute_pass_builder| {
            compute_pass_builder
                .read(radiance_buffer_id, radiance_view_id)
                .read_if(ui_buffer_id, ui_view_id)
                .write(view_target_id, view_view_id)
                .execute(|_, _| {
                    println!("Combine pass execute");
                })
        })
    }
}
