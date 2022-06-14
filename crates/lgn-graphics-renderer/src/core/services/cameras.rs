use lgn_graphics_cgen_runtime::Float4;
use lgn_math::{Mat4, Vec2, Vec4};
use lgn_transform::prelude::GlobalTransform;

use crate::{cgen, components::CameraComponent};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct RenderCamera {
    pub view_transform: GlobalTransform,
    pub projection: Mat4,
    pub culling_planes: [Float4; 6],
}

impl From<(&GlobalTransform, &CameraComponent)> for RenderCamera {
    fn from((_global_transform, camera_component): (&GlobalTransform, &CameraComponent)) -> Self {
        RenderCamera::new(camera_component, 1920.0, 1080.0) // TMP
    }
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
