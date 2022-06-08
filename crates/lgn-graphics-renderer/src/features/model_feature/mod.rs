use crate::core::{
    RenderFeature, RenderLayerId, RenderListCallable, RenderListSlice, RenderListSliceRequirement,
    RenderListSliceTyped, TmpDrawContext, VisibleView,
};

struct KickLayer {
    mat_idx: u32,
}

#[cfg(debug_assertions)]
impl Drop for KickLayer {
    fn drop(&mut self) {
        // println!("KickLayer dropped");
    }
}

impl RenderListCallable for KickLayer {
    fn call(&self, _draw_context: &mut TmpDrawContext) {
        #[cfg(debug_assertions)]
        {
            // println!("KickLayer called: {}", self.mat_idx);
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
        Some(RenderListSliceRequirement::new::<KickLayer>(1))
    }

    fn prepare_render_list(
        &self,
        _view_id: &VisibleView,
        _layer_id: RenderLayerId,
        render_list_slice: RenderListSlice,
    ) {
        let render_list_slice = RenderListSliceTyped::<KickLayer>::new(render_list_slice);

        for writer in render_list_slice.iter() {
            writer.write(0, KickLayer { mat_idx: 0xff });
        }
    }
}
