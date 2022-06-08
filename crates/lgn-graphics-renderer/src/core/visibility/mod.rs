use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bumpalo_herd::Herd;
use lgn_tracing::span_scope;

use crate::components::CameraComponent;
use crate::core::RenderLayerMask;

use super::RenderLayers;

pub struct VisibleView {
    pub render_layer_mask: RenderLayerMask,
}

pub struct VisibilitySet<'a> {
    views: &'a [VisibleView],
}

impl<'a> VisibilitySet<'a> {
    fn new(views: &'a [VisibleView]) -> Self {
        Self { views }
    }

    pub fn views(&self) -> &[VisibleView] {
        self.views
    }
}

pub struct VisibilityContext<'rt> {
    pub herd: &'rt Herd,
    pub bump: &'rt Bump,
    pub camera: &'rt CameraComponent,
    pub render_layers: &'rt RenderLayers,
}

impl<'rt> VisibilityContext<'rt> {
    #[must_use]
    pub fn execute(&self) -> &'rt VisibilitySet<'rt> {
        span_scope!("Visibility");

        let bump = self.bump;

        let mut visible_views = BumpVec::new_in(bump);

        // Push the root view.

        let mut render_layer_mask = RenderLayerMask::default();
        render_layer_mask.add(self.render_layers.get_from_name("DEPTH"));
        render_layer_mask.add(self.render_layers.get_from_name("OPAQUE"));

        visible_views.push(VisibleView { render_layer_mask });

        bump.alloc(VisibilitySet::new(visible_views.into_bump_slice()))
    }
}
