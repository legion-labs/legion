use crate::core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId};

pub struct LightingPass;

impl LightingPass {
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
        ao_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_compute_pass("Lighting", |compute_pass_builder| {
            compute_pass_builder
                .read(gbuffer_view_ids[0], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[1], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[2], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[3], RenderGraphLoadState::Load)
                .read(depth_view_id, RenderGraphLoadState::Load)
                .read(ao_view_id, RenderGraphLoadState::Load)
                .write(radiance_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _| {
                    //println!("LightingPass execute");
                })
        })
    }
}
