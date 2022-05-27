use crate::core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId};

pub struct AlphaBlendedLayerPass;

impl AlphaBlendedLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("AlphaBlendedLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, radiance_view_id, RenderGraphLoadState::Load)
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(|_, _, _| {
                    //println!("AlphaBlendedLayerPass execute");
                })
        })
    }
}
