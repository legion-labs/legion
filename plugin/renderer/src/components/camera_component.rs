use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
};
use lgn_math::{EulerRot, Quat, Vec3};
use lgn_transform::components::Transform;

#[derive(Component)]
pub struct CameraComponent {
    pub speed: f32,
    pub rotation_speed: f32,
}

impl CameraComponent {
    pub fn default_transform() -> Transform {
        let eye = Vec3::new(0.0, 1.0, -2.0);
        let center = Vec3::new(0.0, 0.0, 0.0);

        Transform {
            translation: eye,
            rotation: Quat::from_rotation_arc(
                Vec3::new(0.0, 0.0, -1.0),
                (center - eye).normalize(),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            speed: 2.0,
            rotation_speed: 0.5,
        }
    }
}

pub(crate) fn create_camera(mut commands: Commands<'_, '_>) {
    // camera
    commands
        .spawn()
        .insert(CameraComponent::default())
        .insert(CameraComponent::default_transform());
}

#[derive(Default)]
pub(crate) struct CameraMoving(bool);

pub(crate) fn camera_control(
    mut q_cameras: Query<'_, '_, (&mut CameraComponent, &mut Transform)>,
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

    if q_cameras.is_empty() || !camera_moving.0 {
        return;
    }

    let (mut camera, mut transform) = q_cameras.iter_mut().next().unwrap();
    {
        for keyboard_input_event in keyboard_input_events.iter() {
            if let Some(key_code) = keyboard_input_event.key_code {
                match key_code {
                    KeyCode::W => {
                        let dir = transform.forward();
                        transform.translation += dir * camera.speed / 60.0;
                    }
                    KeyCode::S => {
                        let dir = transform.back();
                        transform.translation += dir * camera.speed / 60.0;
                    }
                    KeyCode::D => {
                        let dir = transform.right();
                        transform.translation += dir * camera.speed / 60.0;
                    }
                    KeyCode::A => {
                        let dir = transform.left();
                        transform.translation += dir * camera.speed / 60.0;
                    }
                    _ => {}
                }
            }
        }

        for mouse_motion_event in mouse_motion_events.iter() {
            let (euler_x, euler_y, euler_z) = transform.rotation.to_euler(EulerRot::XYZ);
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                euler_x + mouse_motion_event.delta.y * camera.rotation_speed / 60.0,
                euler_y - mouse_motion_event.delta.x * camera.rotation_speed / 60.0,
                euler_z,
            );
        }

        for mouse_wheel_event in mouse_wheel_events.iter() {
            camera.speed = (camera.speed * (1.0 + mouse_wheel_event.y * 0.1)).clamp(0.01, 10.0);
        }
    }
}
