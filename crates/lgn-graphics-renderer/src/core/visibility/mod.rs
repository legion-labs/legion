use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bumpalo_herd::Herd;
use lgn_tracing::span_scope;

use crate::components::CameraComponent;

pub type ViewId = u32;
pub type LayerId = u32;

pub struct VisibleView {
    pub id: ViewId,
    pub layers: Vec<LayerId>,
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
}

impl<'rt> VisibilityContext<'rt> {
    #[must_use]
    pub fn execute(&self) -> &'rt VisibilitySet<'rt> {
        span_scope!("Visibility");

        let bump = self.bump;

        let mut visible_views = BumpVec::new_in(bump);
        visible_views.push(VisibleView {
            id: 0,
            layers: Vec::new(),
        });

        bump.alloc(VisibilitySet::new(visible_views.into_bump_slice()))
    }
}
