use std::marker::PhantomData;

use dolly::driver::RigDriver;
use dolly::prelude::{Handedness, Position, Smooth};
use dolly::rig::{CameraRig, RigUpdateParams};
use dolly::transform::Transform;
use lgn_core::Time;
use lgn_ecs::prelude::*;
use lgn_graphics_cgen_runtime::Float4;
use lgn_graphics_data::runtime::CameraSetup;
use lgn_input::gamepad::GamepadButtonType;
use lgn_input::mouse::MouseScrollUnit;
use lgn_input::{
    mouse::{MouseMotion, MouseWheel},
    prelude::{
        Axis, GamepadAxis, GamepadAxisType, GamepadButton, Gamepads, Input, KeyCode, MouseButton,
    },
};
use lgn_math::{Angle, DMat4, EulerRot, Mat3, Mat4, Quat, Vec3, Vec4};
use lgn_transform::components::GlobalTransform;

use crate::UP_VECTOR;

#[derive(Debug)]
pub struct EulerRotator {
    alpha: Angle,
    beta: Angle,
    gamma: Angle,

    euler: EulerRot,
}

impl Default for EulerRotator {
    fn default() -> Self {
        Self::new(EulerRot::YZX)
    }
}

impl EulerRotator {
    pub fn new(euler: EulerRot) -> Self {
        Self {
            alpha: Angle::from_degrees(0.0),
            beta: Angle::from_degrees(0.0),
            gamma: Angle::from_degrees(0.0),
            euler,
        }
    }

    #[must_use]
    pub fn rotation_quat(mut self, rotation: Quat) -> Self {
        self.set_rotation_quat(rotation);
        self
    }

    pub fn rotate(&mut self, alpha: Angle, beta: Angle, gamma: Angle) {
        self.set_rotation_angles(self.alpha + alpha, self.beta + beta, self.gamma + gamma);
    }

    pub fn set_rotation_quat(&mut self, rotation: Quat) {
        let (alpha, beta, gamma) = rotation.to_euler(self.euler);
        self.set_rotation_angles(
            Angle::from_radians(alpha),
            Angle::from_radians(beta),
            Angle::from_radians(gamma),
        );
    }

    fn set_rotation_angles(&mut self, alpha: Angle, beta: Angle, gamma: Angle) {
        self.alpha = Angle::from_radians(alpha.radians() % (std::f32::consts::TAU));
        self.beta = Angle::from_radians(beta.radians() % (std::f32::consts::TAU));
        self.gamma = Angle::from_radians(gamma.radians().clamp(-std::f32::consts::PI, 0.0));
    }
}

impl<H: Handedness> RigDriver<H> for EulerRotator {
    fn update(&mut self, params: RigUpdateParams<'_, H>) -> Transform<H> {
        Transform {
            position: params.parent.position,
            rotation: Quat::from_euler(
                self.euler,
                self.alpha.radians(),
                self.beta.radians(),
                self.gamma.radians(),
            ),
            phantom: PhantomData,
        }
    }
}
#[derive(Component)]
pub struct CameraComponent {
    camera_rig: CameraRig,
    speed: f32,
    rotation_speed: Angle,
    setup: CameraSetup,
    fov_y: Angle,
    z_near: f32,
    z_far: f32,
}

impl CameraComponent {
    pub fn view_transform(&self) -> GlobalTransform {
        let eye = self.camera_rig.final_transform.position.as_dvec3();
        let forward = self.camera_rig.final_transform.forward().as_dvec3();

        let view_matrix = DMat4::look_at_rh(eye, eye + forward, UP_VECTOR.as_dvec3());
        let (_scale, rotation, translation) = view_matrix.to_scale_rotation_translation();

        let mut view_transform = GlobalTransform::identity();
        view_transform.translation = translation.as_vec3();
        view_transform.rotation = rotation.as_f32();

        view_transform
    }

    pub fn build_projection(&self, width: f32, height: f32) -> Mat4 {
        let aspect_ratio = width / height;
        Mat4::perspective_infinite_reverse_rh(self.fov_y.radians(), aspect_ratio, self.z_near)
    }

    pub fn build_culling_planes(&self, aspect_ratio: f32) -> [Float4; 6] {
        let eye = self.camera_rig.final_transform.position;
        let forward = self.camera_rig.final_transform.forward();
        let up = self.camera_rig.final_transform.up();
        let right = self.camera_rig.final_transform.right();

        let half_v_side = self.z_far * (self.fov_y.radians() * 0.5).tan();
        let half_h_side = half_v_side * aspect_ratio;

        let near_face_point = eye + forward * self.z_near;
        let near_normal = -forward;
        let near_plane: Float4 =
            Vec4::from((near_normal, -near_normal.dot(near_face_point))).into();

        let far_face_point = eye + forward * self.z_far;
        let far_normal = forward;
        let far_plane: Float4 = Vec4::from((far_normal, -far_normal.dot(far_face_point))).into();

        let front_mult_far = self.z_far * forward;

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

    pub fn position(&self) -> Vec3 {
        self.camera_rig.final_transform.position
    }

    pub fn rotation(&self) -> Quat {
        self.camera_rig.final_transform.rotation
    }

    pub fn fov_y(&self) -> Angle {
        self.fov_y
    }

    pub fn z_near(&self) -> f32 {
        self.z_near
    }

    pub fn z_far(&self) -> f32 {
        self.z_far
    }

    fn build_rig(setup: &CameraSetup) -> CameraRig {
        let forward = (setup.look_at - setup.eye).normalize();
        let forward_dot = forward.dot(UP_VECTOR);
        let right = if (forward_dot - 1.0).abs() < std::f32::EPSILON {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (forward_dot + 1.0).abs() < std::f32::EPSILON {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            forward.cross(UP_VECTOR).normalize()
        };
        let up = right.cross(forward);
        let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, -forward));

        CameraRig::builder()
            .with(Position::new(setup.eye))
            .with(EulerRotator::new(EulerRot::YZX).rotation_quat(rotation))
            .with(Smooth::new_position_rotation(0.2, 0.2))
            .build()
    }

    fn reset(&mut self) {
        self.camera_rig = Self::build_rig(&self.setup);
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        let setup = CameraSetup {
            eye: Vec3::new(0.0, 2.0, 1.0),
            look_at: Vec3::ZERO,
        };

        Self {
            camera_rig: Self::build_rig(&setup),
            speed: 2.5,
            rotation_speed: Angle::from_degrees(40.0),
            setup,
            fov_y: Angle::from_radians(std::f32::consts::FRAC_PI_4),
            z_near: 0.01,
            z_far: 100.0,
        }
    }
}

pub(crate) fn create_camera(mut commands: Commands<'_, '_>) {
    commands.spawn().insert(CameraComponent::default());
}

#[derive(Component, Default)]
pub(crate) struct CameraSetupApplied(); // marker component

pub(crate) fn apply_camera_setups(
    camera_setups: Query<'_, '_, (Entity, &CameraSetup), Without<CameraSetupApplied>>,
    mut cameras: Query<'_, '_, &mut CameraComponent>,
    mut commands: Commands<'_, '_>,
) {
    for (entity, setup) in camera_setups.iter() {
        if let Some(mut camera) = cameras.iter_mut().next() {
            let camera = camera.as_mut();
            camera.setup = setup.clone();
            camera.reset();
        }
        commands
            .entity(entity)
            .insert(CameraSetupApplied::default());
    }

    drop(camera_setups);
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn camera_control(
    mut cameras_query: Query<'_, '_, &mut CameraComponent>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mouse_buttons: Res<'_, Input<MouseButton>>,
    keys: Res<'_, Input<KeyCode>>,
    gamepads: Res<'_, Gamepads>,
    gamepad_axes: Res<'_, Axis<GamepadAxis>>,
    gamepad_buttons: Res<'_, Input<GamepadButton>>,

    time: Res<'_, Time>,
) {
    if cameras_query.is_empty() {
        return;
    }
    // Need to associate inputs with window/camera... we don''t have that for now
    for mut camera in cameras_query.iter_mut() {
        let camera = camera.as_mut();

        if keys.pressed(KeyCode::Z)
            && !keys.any_pressed([
                KeyCode::LShift,
                KeyCode::RShift,
                KeyCode::LControl,
                KeyCode::RControl,
            ])
        {
            camera.reset();
            continue;
        }

        let gamepad = gamepads.iter().copied().find(|gamepad| {
            if let Some(left_x) =
                gamepad_axes.get(GamepadAxis(*gamepad, GamepadAxisType::LeftStickX))
            {
                if left_x.abs() > 0.01 {
                    true
                } else if let Some(left_y) =
                    gamepad_axes.get(GamepadAxis(*gamepad, GamepadAxisType::LeftStickY))
                {
                    left_y.abs() > 0.01
                } else {
                    false
                }
            } else {
                false
            }
        });

        if gamepad.is_some() || mouse_buttons.pressed(MouseButton::Right) {
            let mut camera_translation_change = Vec3::ZERO;

            if keys.pressed(KeyCode::W) {
                camera_translation_change += camera.camera_rig.final_transform.forward();
            }
            if keys.pressed(KeyCode::S) {
                camera_translation_change -= camera.camera_rig.final_transform.forward();
            }
            if keys.pressed(KeyCode::A) {
                camera_translation_change -= camera.camera_rig.final_transform.right();
            }
            if keys.pressed(KeyCode::D) {
                camera_translation_change += camera.camera_rig.final_transform.right();
            }

            if let Some(gamepad) = gamepad {
                if let Some(left_x) =
                    gamepad_axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
                {
                    camera_translation_change += left_x * camera.camera_rig.final_transform.right();
                }

                if let Some(left_y) =
                    gamepad_axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
                {
                    camera_translation_change +=
                        left_y * camera.camera_rig.final_transform.forward();
                }
            }

            let mut speed = camera.speed.exp();
            if let Some(gamepad) = gamepad {
                if gamepad_buttons.pressed(GamepadButton(gamepad, GamepadButtonType::RightTrigger2))
                {
                    speed *= 5.0;
                }
            }
            if keys.pressed(KeyCode::LShift) {
                speed *= 2.0;
            }
            camera_translation_change *= speed * time.delta_seconds();

            camera
                .camera_rig
                .driver_mut::<Position>()
                .translate(camera_translation_change);

            let rotation_speed = camera.rotation_speed;
            let camera_driver = camera.camera_rig.driver_mut::<EulerRotator>();
            for mouse_motion_event in mouse_motion_events.iter() {
                let rotation = (rotation_speed.degrees() * time.delta_seconds()).min(10.0); // clamping rotation speed for when it's laggy
                camera_driver.rotate(
                    Angle::from_degrees(0.0),
                    Angle::from_degrees(mouse_motion_event.delta.x * rotation),
                    Angle::from_degrees(-mouse_motion_event.delta.y * rotation),
                );
            }
            for mouse_wheel_event in mouse_wheel_events.iter() {
                // Different signs on Line and Pixel is correct. Line returns positive values when scrolling up
                // and pixels return negative values
                let speed_change = match mouse_wheel_event.unit {
                    MouseScrollUnit::Line => mouse_wheel_event.y * 0.1,
                    // Last time I tested one segment of a wheel would yield 250 pixels,
                    MouseScrollUnit::Pixel => -mouse_wheel_event.y / 1000.0,
                };
                camera.speed = (camera.speed + speed_change).clamp(0.0, 10.0);
            }
        }
        camera.camera_rig.update(time.delta_seconds());
    }
}
