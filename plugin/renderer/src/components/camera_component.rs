use dolly::prelude::{Position, Smooth, YawPitch};
use dolly::rig::CameraRig;
use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
};
use lgn_math::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::cgen;

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
        let up = Vec3::new(0.0, 1.0, 0.0); // self.camera_rig.final_transform.up();
        let view_matrix = Mat4::look_at_lh(eye, center, up);

        let fov_y_radians: f32 = 45.0;
        let aspect_ratio = width / height;
        let z_near: f32 = 0.01;
        let z_far: f32 = 100.0;
        let projection_matrix = Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far);

        (view_matrix, projection_matrix)
    }

    pub fn tmp_build_view_data(
        &self,
        output_width: f32,
        output_height: f32,
        picking_width: f32,
        picking_height: f32,
        cursor_x: f32,
        cursor_y: f32,
    ) -> cgen::cgen_type::ViewData {
        let (view_matrix, projection_matrix) =
            self.build_view_projection(output_width, output_height);
        let view_proj_matrix = projection_matrix * view_matrix;

        let mut camera_props = cgen::cgen_type::ViewData::default();

        camera_props.set_view(view_matrix.into());
        camera_props.set_inv_view(view_matrix.inverse().into());
        camera_props.set_projection(projection_matrix.into());
        camera_props.set_inv_projection(projection_matrix.inverse().into());
        camera_props.set_projection_view(view_proj_matrix.into());
        camera_props.set_inv_projection_view(view_proj_matrix.inverse().into());
        camera_props.set_screen_size(
            Vec4::new(
                picking_width,
                picking_height,
                1.0 / picking_width,
                1.0 / picking_height,
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
        let center = Vec3::new(0.0, 0.0, 0.0);

        let forward = (center - eye).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, -forward));

        let camera_rig = CameraRig::builder()
            .with(Position::new(eye))
            .with(YawPitch::new().rotation_quat(rotation))
            .with(Smooth::new_position_rotation(1.1, 1.1))
            .build();

        Self {
            camera_rig,
            speed: 5.0,
            rotation_speed: 45.0,
        }
    }
}

pub(crate) fn create_camera(mut commands: Commands<'_, '_>) {
    commands.spawn().insert(CameraComponent::default());
}

#[derive(Default)]
pub(crate) struct CameraMoving(bool);

pub(crate) fn camera_control(
    mut q_cameras: Query<'_, '_, &mut CameraComponent>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mut mouse_button_input_events: EventReader<'_, '_, MouseButtonInput>,
    mut camera_moving: Local<'_, CameraMoving>,
) {
    for mouse_button_input_event in mouse_button_input_events.iter() {
        if mouse_button_input_event.button == MouseButton::Right {
            camera_moving.0 = mouse_button_input_event.state.is_pressed();
        }
    }

    const FRAME_TIME: f32 = 1.0 / 60.0;

    if q_cameras.is_empty() {
        return;
    }

    let mut camera = q_cameras.iter_mut().next().unwrap();

    if !camera_moving.0 {
        camera.camera_rig.update(FRAME_TIME);
        return;
    }

    let mut camera_translation_change = Vec3::ZERO;

    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            match key_code {
                KeyCode::W => {
                    let dir = camera.camera_rig.final_transform.forward();
                    camera_translation_change += dir * camera.speed / 60.0;
                }
                KeyCode::S => {
                    let dir = -camera.camera_rig.final_transform.forward();
                    camera_translation_change += dir * camera.speed / 60.0;
                }
                KeyCode::D => {
                    let dir = -camera.camera_rig.final_transform.right();
                    camera_translation_change += dir * camera.speed / 60.0;
                }
                KeyCode::A => {
                    let dir = camera.camera_rig.final_transform.right();
                    camera_translation_change += dir * camera.speed / 60.0;
                }
                _ => {}
            }
        }
    }

    camera
        .camera_rig
        .driver_mut::<Position>()
        .translate(camera_translation_change);

    let rotation_speed = camera.rotation_speed;
    let camera_driver = camera.camera_rig.driver_mut::<YawPitch>();
    for mouse_motion_event in mouse_motion_events.iter() {
        camera_driver.rotate_yaw_pitch(
            mouse_motion_event.delta.x * rotation_speed / 60.0,
            -mouse_motion_event.delta.y * rotation_speed / 60.0,
        );
    }
    for mouse_wheel_event in mouse_wheel_events.iter() {
        camera.speed = (camera.speed * (1.0 + mouse_wheel_event.y * 0.1)).clamp(0.01, 10.0);
    }

    camera.camera_rig.update(FRAME_TIME);
}
