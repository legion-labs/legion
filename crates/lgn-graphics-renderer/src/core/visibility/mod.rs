use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bumpalo_herd::Herd;
use lgn_graphics_cgen_runtime::Float4;
use lgn_math::{Mat4, Vec2, Vec4};
use lgn_tracing::span_scope;
use lgn_transform::prelude::GlobalTransform;

use crate::cgen;
use crate::components::CameraComponent;
use crate::core::RenderLayerMask;

use super::RenderLayers;

#[derive(Clone, Copy)]
pub struct RenderCamera {
    pub view_transform: GlobalTransform,
    pub projection: Mat4,
    pub culling_planes: [Float4; 6],
}

impl RenderCamera {
    pub fn new(camera_component: &CameraComponent, width: f32, height: f32) -> Self {
        Self {
            view_transform: camera_component.view_transform(),
            projection: camera_component.build_projection(width, height),
            culling_planes: camera_component.build_culling_planes(width / height),
        }
    }

    pub fn tmp_build_view_data(
        &self,
        pixel_width: f32,
        pixel_height: f32,
        logical_width: f32,
        logical_height: f32,
        cursor_x: f32,
        cursor_y: f32,
    ) -> cgen::cgen_type::ViewData {
        let mut camera_props = cgen::cgen_type::ViewData::default();

        camera_props.set_camera_translation(self.view_transform.translation.into());
        camera_props.set_camera_rotation(Vec4::from(self.view_transform.rotation).into());
        camera_props.set_projection(self.projection.into());
        camera_props.set_culling_planes(self.culling_planes);
        camera_props.set_pixel_size(
            Vec4::new(
                pixel_width,
                pixel_height,
                1.0 / pixel_width,
                1.0 / pixel_height,
            )
            .into(),
        );
        camera_props.set_logical_size(
            Vec4::new(
                logical_width,
                logical_height,
                1.0 / logical_width,
                1.0 / logical_height,
            )
            .into(),
        );
        camera_props.set_cursor_pos(Vec2::new(cursor_x, cursor_y).into());

        camera_props
    }
}

pub struct VisibleView {
    pub render_layer_mask: RenderLayerMask,
    pub render_camera: RenderCamera,
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
    pub render_camera: RenderCamera,
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

        visible_views.push(VisibleView {
            render_layer_mask,
            render_camera: self.render_camera,
        });

        bump.alloc(VisibilitySet::new(visible_views.into_bump_slice()))
    }
}
