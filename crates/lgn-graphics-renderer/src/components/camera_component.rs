use std::marker::PhantomData;

use dolly::driver::RigDriver;
use dolly::prelude::{Handedness, Position, RightHanded, Smooth};
use dolly::rig::{CameraRig, RigUpdateParams};
use dolly::transform::Transform;
use lgn_core::Time;
use lgn_ecs::prelude::*;
use lgn_graphics_data::runtime::CameraSetup;
use lgn_input::gamepad::GamepadButtonType;
use lgn_input::mouse::MouseScrollUnit;
use lgn_input::{
    mouse::{MouseMotion, MouseWheel},
    prelude::{
        Axis, GamepadAxis, GamepadAxisType, GamepadButton, Gamepads, Input, KeyCode, MouseButton,
    },
};
use lgn_math::{Angle, EulerRot, Mat3, Quat, Vec3};
use lgn_transform::components::GlobalTransform;
use lgn_utils::HashMap;

use crate::core::{PrimaryTableView, RenderCamera, RenderObjectId};
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
    render_object_id: Option<RenderObjectId>,
}

impl CameraComponent {
    pub fn position(&self) -> Vec3 {
        self.camera_rig.final_transform.position
    }

    pub fn rotation(&self) -> Quat {
        self.camera_rig.final_transform.rotation
    }

    pub fn final_transform(&self) -> Transform<RightHanded> {
        self.camera_rig.final_transform
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

    pub fn render_object_id(&self) -> Option<RenderObjectId> {
        self.render_object_id
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
            render_object_id: None,
        }
    }
}

pub(crate) fn tmp_create_camera(mut commands: Commands<'_, '_>) {
    commands
        .spawn()
        .insert_bundle((GlobalTransform::default(), CameraComponent::default()));
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

pub(crate) struct EcsToRenderCamera {
    view: PrimaryTableView<RenderCamera>,
    map: HashMap<Entity, RenderObjectId>,
}

impl EcsToRenderCamera {
    pub fn new(view: PrimaryTableView<RenderCamera>) -> Self {
        Self {
            map: HashMap::new(),
            view,
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_camera_components(
    mut queries: ParamSet<
        '_,
        '_,
        (
            Query<
                '_,
                '_,
                (&GlobalTransform, &mut CameraComponent),
                Or<(Changed<GlobalTransform>, Changed<CameraComponent>)>,
            >,
            Query<'_, '_, (Entity, &CameraComponent), Added<CameraComponent>>,
        ),
    >,

    q_removals: RemovedComponents<'_, CameraComponent>,
    mut ecs_to_render: ResMut<'_, EcsToRenderCamera>,
) {
    // Base path. Can be simplfied more by having access to the data of removed components
    {
        let mut writer = ecs_to_render.view.writer();

        for e in q_removals.iter() {
            let render_object_id = ecs_to_render.map.get(&e);
            if let Some(render_object_id) = render_object_id {
                writer.remove(*render_object_id);
            }
        }

        for (transform, mut camera) in queries.p0().iter_mut() {
            if let Some(render_object_id) = camera.render_object_id {
                writer.update(render_object_id, (transform, camera.as_ref()).into());
            } else {
                camera.render_object_id = Some(writer.insert((transform, camera.as_ref()).into()));
            };
        }
    }
    // Update map because of removed components
    {
        let map = &mut ecs_to_render.map;

        for e in q_removals.iter() {
            map.remove(&e);
        }

        for (e, visual) in queries.p1().iter() {
            map.insert(e, visual.render_object_id.unwrap());
        }
    }
}
