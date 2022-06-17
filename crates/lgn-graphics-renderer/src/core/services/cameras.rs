use lgn_graphics_cgen_runtime::Float4;
use lgn_math::{Angle, DMat4, Mat4, Vec2, Vec4};
use lgn_transform::prelude::GlobalTransform;

use crate::{cgen, components::CameraComponent, UP_VECTOR};

pub fn view_transform(camera_transform: &GlobalTransform) -> GlobalTransform {
    let eye = camera_transform.translation.as_dvec3();
    let forward = camera_transform.forward().as_dvec3();

    let view_matrix = DMat4::look_at_rh(eye, eye + forward, camera_transform.up().as_dvec3());
    let (_scale, rotation, translation) = view_matrix.to_scale_rotation_translation();

    let mut view_transform = GlobalTransform::identity();
    view_transform.translation = translation.as_vec3();
    view_transform.rotation = rotation.as_f32();

    view_transform
}

pub fn build_projection(width: f32, height: f32, fov_y: Angle, z_near: f32) -> Mat4 {
    let aspect_ratio = width / height;
    Mat4::perspective_infinite_reverse_rh(fov_y.radians(), aspect_ratio, z_near)
}

pub fn build_culling_planes(
    camera_transform: &GlobalTransform,
    aspect_ratio: f32,
    fov_y: Angle,
    z_near: f32,
    z_far: f32,
) -> [Float4; 6] {
    let eye = camera_transform.translation;
    let forward = camera_transform.forward();
    let up = camera_transform.up();
    let right = camera_transform.right();

    let half_v_side = z_far * (fov_y.radians() * 0.5).tan();
    let half_h_side = half_v_side * aspect_ratio;

    let near_face_point = eye + forward * z_near;
    let near_normal = -forward;
    let near_plane: Float4 = Vec4::from((near_normal, -near_normal.dot(near_face_point))).into();

    let far_face_point = eye + forward * z_far;
    let far_normal = forward;
    let far_plane: Float4 = Vec4::from((far_normal, -far_normal.dot(far_face_point))).into();

    let front_mult_far = z_far * forward;

    let right_side = front_mult_far - right * half_h_side;
    let right_normal = up.cross(right_side).normalize();
    let right_plane: Float4 = Vec4::from((right_normal, -right_normal.dot(eye))).into();

    let left_side = front_mult_far + right * half_h_side;
    let left_normal = left_side.cross(up).normalize();
    let left_plane: Float4 = Vec4::from((left_normal, -left_normal.dot(eye))).into();

    let top_side = front_mult_far - up * half_v_side;
    let top_normal = top_side.cross(right).normalize();
    let top_plane: Float4 = Vec4::from((top_normal, -top_normal.dot(eye))).into();

    let bottom_side = front_mult_far + up * half_v_side;
    let bottom_normal = right.cross(bottom_side).normalize();
    let bottom_plane: Float4 = Vec4::from((bottom_normal, -bottom_normal.dot(eye))).into();

    [
        near_plane,
        far_plane,
        right_plane,
        left_plane,
        top_plane,
        bottom_plane,
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct RenderCamera {
    pub transform: GlobalTransform,
    fov_y: Angle,
    z_near: f32,
    z_far: f32,
}

impl From<(&GlobalTransform, &CameraComponent)> for RenderCamera {
    fn from((global_transform, camera_component): (&GlobalTransform, &CameraComponent)) -> Self {
        RenderCamera::new(camera_component, global_transform)
    }
}

impl RenderCamera {
    pub fn new(camera_component: &CameraComponent, transform: &GlobalTransform) -> Self {
        Self {
            transform: *transform,
            fov_y: camera_component.fov_y(),
            z_near: camera_component.z_near(),
            z_far: camera_component.z_far(),
        }
    }

    pub fn view_transform(&self) -> GlobalTransform {
        view_transform(&self.transform)
    }

    pub fn build_projection(&self, width: f32, height: f32) -> Mat4 {
        build_projection(width, height, self.fov_y, self.z_near)
    }

    pub fn build_culling_planes(&self, aspect_ratio: f32) -> [Float4; 6] {
        build_culling_planes(
            &self.transform,
            aspect_ratio,
            self.fov_y,
            self.z_near,
            self.z_far,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tmp_build_view_data(
        &self,
        pixel_width: f32,
        pixel_height: f32,
        logical_width: f32,
        logical_height: f32,
        cursor_x: f32,
        cursor_y: f32,
    ) -> cgen::cgen_type::ViewData {
        let view_transform = self.view_transform();
        let projection = self.build_projection(pixel_width, pixel_height);
        let culling_planes = self.build_culling_planes(pixel_width / pixel_height);

        let mut camera_props = cgen::cgen_type::ViewData::default();

        camera_props.set_camera_translation(view_transform.translation.into());
        camera_props.set_camera_rotation(Vec4::from(view_transform.rotation).into());
        camera_props.set_projection(projection.into());
        camera_props.set_culling_planes(culling_planes);
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
