use crate::core::{
    RenderFeature, RenderLayerId, RenderListCallable, RenderListSlice, RenderListSliceRequirement,
    RenderListSliceTyped, TmpDrawContext, VisibleView,
};

#[allow(dead_code)]
struct TmpKickLayer {
    render_layer_id: RenderLayerId,
}

#[cfg(debug_assertions)]
impl Drop for TmpKickLayer {
    fn drop(&mut self) {
        // println!("TmpKickLayer dropped");
    }
}

impl RenderListCallable for TmpKickLayer {
    fn call(&self, _draw_context: &mut TmpDrawContext) {
        #[cfg(debug_assertions)]
        {
            // println!("TmpKickLayer called: {}", self.render_layer_id);
        }
    }
}

pub struct ModelFeature {}

impl ModelFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderFeature for ModelFeature {
    fn get_render_list_requirement(
        &self,
        _view_id: &VisibleView,
        _layer_id: RenderLayerId,
    ) -> Option<RenderListSliceRequirement> {
        Some(RenderListSliceRequirement::new::<TmpKickLayer>(1))
    }

    fn prepare_render_list(
        &self,
        _view_id: &VisibleView,
        render_layer_id: RenderLayerId,
        render_list_slice: RenderListSlice,
    ) {
        let render_list_slice = RenderListSliceTyped::<TmpKickLayer>::new(render_list_slice);

        for writer in render_list_slice.iter() {
            writer.write(0, TmpKickLayer { render_layer_id });
        }
    }
}
