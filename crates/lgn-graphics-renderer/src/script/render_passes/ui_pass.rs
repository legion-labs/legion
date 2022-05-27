use crate::core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId};

pub struct UiPass;

impl UiPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        ui_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("UI", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, ui_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _| {
                    //println!("UiPass execute");
                })
        })
    }
}
