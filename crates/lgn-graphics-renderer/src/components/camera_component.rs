use dolly::prelude::{Position, Smooth, YawPitch};
use dolly::rig::CameraRig;
use lgn_core::Time;
use lgn_ecs::prelude::*;
use lgn_graphics_cgen_runtime::Float4;
use lgn_input::Input;
use lgn_input::{
    keyboard::KeyCode,
    mouse::{MouseButton, MouseMotion, MouseWheel},
};
use lgn_math::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{cgen, UP_VECTOR};

#[derive(Component)]
pub struct CameraComponent {
    pub camera_rig: CameraRig,
    pub speed: f32,
    pub rotation_speed: f32,
}

impl CameraComponent {
    pub fn build_view_projection(&self, width: f32, height: f32) -> (Mat4, Mat4) {
        let eye = self.camera_rig.final_transform.position;
        let center = eye + self.camera_rig.final_transform.forward();
        let up = UP_VECTOR; // self.camera_rig.final_transform.up();
        let view_matrix = Mat4::look_at_lh(eye, center, up);

        let fov_y_radians: f32 = 45.0;
        let aspect_ratio = width / height;
        let z_near: f32 = 0.01;
        let z_far: f32 = 500.0;
        let projection_matrix = Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

        (view_matrix, projection_matrix)
    }

    pub fn build_culling_planes(&self, aspect_ratio: f32) -> [Float4; 6] {
        let fov_y_radians: f32 = 45.0;
        let z_near: f32 = 0.01;
        let z_far: f32 = 500.0;

        let eye = self.camera_rig.final_transform.position;
        let forward = self.camera_rig.final_transform.forward();
        let up = self.camera_rig.final_transform.up();
        let right = self.camera_rig.final_transform.right();

        let half_v_side = z_far * (fov_y_radians * 0.5).tan();
        let half_h_side = half_v_side * aspect_ratio;

        let near_face_point = eye + forward * z_near;
        let near_normal = -forward;
        let near_plane: Float4 =
            Vec4::from((near_normal, -near_normal.dot(near_face_point))).into();

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

    pub fn tmp_build_view_data(
        &self,
        pixel_width: f32,
        pixel_height: f32,
        logical_width: f32,
        logical_height: f32,
        cursor_x: f32,
        cursor_y: f32,
    ) -> cgen::cgen_type::ViewData {
        let (view_matrix, projection_matrix) =
            self.build_view_projection(pixel_width, pixel_height);
        let view_proj_matrix = projection_matrix * view_matrix;

        let mut camera_props = cgen::cgen_type::ViewData::default();

        camera_props.set_view(view_matrix.into());
        camera_props.set_inv_view(view_matrix.inverse().into());
        camera_props.set_projection(projection_matrix.into());
        camera_props.set_inv_projection(projection_matrix.inverse().into());
        camera_props.set_projection_view(view_proj_matrix.into());
        camera_props.set_inv_projection_view(view_proj_matrix.inverse().into());
        camera_props.set_culling_planes(self.build_culling_planes(pixel_width / pixel_height));
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

impl Default for CameraComponent {
    fn default() -> Self {
        let eye = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(2.0, 2.0, 0.0);

        let forward = (center - eye).normalize();
        let right = forward.cross(UP_VECTOR).normalize();
        let up = right.cross(forward);
        let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, -forward));

        let camera_rig = CameraRig::builder()
            .with(Position::new(eye))
            .with(YawPitch::new().rotation_quat(rotation))
            .with(Smooth::new_position_rotation(0.2, 0.2))
            .build();

        Self {
            camera_rig,
            speed: 2.5,
            rotation_speed: 40.0,
        }
    }
}

pub(crate) fn create_camera(mut commands: Commands<'_, '_>) {
    commands.spawn().insert(CameraComponent::default());
}

#[derive(Default)]
pub(crate) struct CameraMoving(bool);

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn camera_control(
    mut cameras_query: Query<'_, '_, &mut CameraComponent>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mouse_buttons: Res<'_, Input<MouseButton>>,
    keys: Res<'_, Input<KeyCode>>,
    time: Res<'_, Time>,
) {
    if cameras_query.is_empty() {
        return;
    }
    // Need to associate inputs with window/camera... we don''t have that for now
    for mut camera in cameras_query.iter_mut() {
        let camera = camera.as_mut();
        if !mouse_buttons.pressed(MouseButton::Right) {
            camera.camera_rig.update(time.delta_seconds());
            continue;
        }
        let mut camera_translation_change = Vec3::ZERO;
        if keys.pressed(KeyCode::W) {
            camera_translation_change += camera.camera_rig.final_transform.forward();
        }
        if keys.pressed(KeyCode::S) {
            camera_translation_change -= camera.camera_rig.final_transform.forward();
        }
        if keys.pressed(KeyCode::A) {
            camera_translation_change += camera.camera_rig.final_transform.right();
        }
        if keys.pressed(KeyCode::D) {
            camera_translation_change -= camera.camera_rig.final_transform.right();
        }
        let mut speed = camera.speed;
        if keys.pressed(KeyCode::LShift) {
            speed *= 2.0;
        }
        camera_translation_change *= speed * time.delta_seconds();

        camera
            .camera_rig
            .driver_mut::<Position>()
            .translate(camera_translation_change);

        let rotation_speed = camera.rotation_speed;
        let camera_driver = camera.camera_rig.driver_mut::<YawPitch>();
        for mouse_motion_event in mouse_motion_events.iter() {
            camera_driver.rotate_yaw_pitch(
                mouse_motion_event.delta.x * rotation_speed * time.delta_seconds(),
                -mouse_motion_event.delta.y * rotation_speed * time.delta_seconds(),
            );
        }
        for mouse_wheel_event in mouse_wheel_events.iter() {
            camera.speed = (camera.speed * (1.0 + mouse_wheel_event.y * 0.1)).clamp(0.01, 10.0);
        }

        camera.camera_rig.update(time.delta_seconds());
    }
}
