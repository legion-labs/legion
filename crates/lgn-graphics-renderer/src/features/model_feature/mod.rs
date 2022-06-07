use super::RenderFeature;

pub struct ModelFeature {}

impl ModelFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderFeature for ModelFeature {
    fn get_render_list_requirement(
        &self,
        _view_id: crate::core::ViewId,
        _layer_id: crate::core::LayerId,
    ) -> Option<crate::core::Requirement> {
        todo!()
    }

    fn prepare_render_list(
        &self,
        _view_id: crate::core::ViewId,
        _layer_id: crate::core::LayerId,
        builder: crate::core::RenderListSlice,
    ) {
        todo!()
    }
}
