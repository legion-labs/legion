use crate::gpu_renderer::RenderLayerId;

use super::{PrepareRenderListContext, RenderFeature, RenderListRequirement, VisibleView};

pub struct ModelFeature {}

impl ModelFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderFeature for ModelFeature {
    fn get_render_list_requirement(
        &self,
        _: &PrepareRenderListContext<'_>,
        _: &VisibleView,
        _: RenderLayerId,
    ) -> Option<RenderListRequirement> {
        let render_item_count = 1024;
        Some(RenderListRequirement {
            render_item_count,
            attached_data_size: 0,
        })
    }
}
