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
use lgn_math::{Angle, DMat4, EulerRot, Mat3, Quat, Vec3};
use lgn_transform::components::GlobalTransform;
use lgn_utils::HashMap;

use crate::core::{PrimaryTableView, RenderCamera, RenderObjectId};

pub struct CameraOptions {
    setup: CameraSetup,
    speed: f32,
    rotation_speed: Angle,
}

impl CameraOptions {
    fn new(
        setup: &CameraSetup,
        camera_transform: &mut GlobalTransform,
        speed: f32,
        rotation_speed: Angle,
    ) -> Self {
        let camera_options = Self {
            setup: setup.clone(),
            speed,
            rotation_speed,
        };
        camera_options.setup_camera_transform(camera_transform);
        camera_options

        //let forward = (setup.look_at - setup.eye).normalize();
        //
        //let (yaw, pitch, _roll) = vector_to_euler(forward);
        //
        //assert!(yaw <= std::f32::consts::TAU);
        //assert!(yaw >= 0.0);
        //
        //assert!(pitch <= std::f32::consts::PI);
        //assert!(pitch >= -std::f32::consts::PI);
        //
        //CameraRig::builder()
        //    .with(Position::new(setup.eye))
        //    .with(
        //        YawPitch::new()
        //            .yaw_degrees(yaw.to_degrees())
        //            .pitch_degrees(pitch.to_degrees()),
        //    )
        //    .with(Smooth::new_position_rotation(0.2, 0.2))
        //    .build()
    }

    fn reset(&self, camera_transform: &mut GlobalTransform) {
        self.setup_camera_transform(camera_transform);
    }

    pub fn view_transform(camera_transform: &GlobalTransform) -> GlobalTransform {
        let eye = camera_transform.translation.as_dvec3();
        let forward = camera_transform.forward().as_dvec3();

        let world2view = DMat4::look_at_rh(eye, eye + forward, camera_transform.up().as_dvec3());
        let (_scale, rotation, translation) = world2view.to_scale_rotation_translation();

        let mut view_transform = GlobalTransform::identity();
        view_transform.translation = translation.as_vec3();
        view_transform.rotation = rotation.as_f32();

        view_transform
    }

    fn setup_camera_transform(&self, camera_transform: &mut GlobalTransform) {
        let forward = self.setup.look_at - self.setup.eye;
        let (yaw, pitch) = dir_to_yaw_pitch(forward);
        camera_transform.rotation = Quat::from_euler(EulerRot::YZX, 0.0, yaw, pitch);
        camera_transform.translation = self.setup.eye;
    }
}

impl Default for CameraOptions {
    fn default() -> Self {
        let setup = CameraSetup {
            eye: Vec3::new(0.0, -2.0, 1.0),
            look_at: Vec3::ZERO,
        };

        Self {
            speed: 2.5,
            rotation_speed: Angle::from_degrees(40.0),
            setup,
        }
    }
}

fn dir_to_yaw_pitch(dir: Vec3) -> (f32, f32) {
    let yaw = std::f32::consts::PI - (dir.x).atan2(-dir.y);
    let dir_no_yaw = Mat3::from_rotation_z(yaw) * dir;
    let pitch = (dir_no_yaw.y).atan2(-dir_no_yaw.z);
    (yaw, pitch)
}

#[derive(Component)]
pub struct CameraComponent {
    fov_y: Angle,
    z_near: f32,
    z_far: f32,
    render_object_id: Option<RenderObjectId>,
}

impl CameraComponent {
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
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_y: Angle::from_radians(std::f32::consts::FRAC_PI_4),
            z_near: 0.01,
            z_far: 100.0,
            render_object_id: None,
        }
    }
}

pub(crate) fn tmp_create_camera(
    mut commands: Commands<'_, '_>,
    camera_options: Res<'_, CameraOptions>,
) {
    let mut camera_transform = GlobalTransform::default();
    camera_options.reset(&mut camera_transform);
    commands
        .spawn()
        .insert_bundle((camera_transform, CameraComponent::default()));
}

#[derive(Component, Default)]
pub(crate) struct CameraSetupApplied(); // marker component

pub(crate) fn apply_camera_setups(
    camera_setups: Query<'_, '_, (Entity, &CameraSetup), Without<CameraSetupApplied>>,
    mut cameras: Query<'_, '_, &mut GlobalTransform, With<CameraComponent>>,
    mut commands: Commands<'_, '_>,
    mut camera_options: ResMut<'_, CameraOptions>,
) {
    for (entity, setup) in camera_setups.iter() {
        if let Some(mut transform) = cameras.iter_mut().next() {
            camera_options.setup = setup.clone();
            camera_options.reset(transform.as_mut());
        }
        commands
            .entity(entity)
            .insert(CameraSetupApplied::default());
    }

    drop(camera_setups);
}

pub struct CameraRotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraRotation {
    pub fn rotate_yaw_pitch(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw;
        self.pitch += pitch;
        self.yaw = (self.yaw + yaw) % std::f32::consts::TAU;
        self.pitch = (self.pitch + pitch).clamp(0.0, std::f32::consts::PI);
    }
}

impl Default for CameraRotation {
    fn default() -> Self {
        CameraRotation {
            yaw: std::f32::consts::PI,
            pitch: std::f32::consts::FRAC_PI_2,
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn camera_control(
    mut cameras_query: Query<'_, '_, &mut GlobalTransform, With<CameraComponent>>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mouse_buttons: Res<'_, Input<MouseButton>>,
    keys: Res<'_, Input<KeyCode>>,
    gamepads: Res<'_, Gamepads>,
    gamepad_axes: Res<'_, Axis<GamepadAxis>>,
    gamepad_buttons: Res<'_, Input<GamepadButton>>,
    mut camera_options: ResMut<'_, CameraOptions>,
    mut camera_rotation: Local<'_, CameraRotation>,

    time: Res<'_, Time>,
) {
    if cameras_query.is_empty() {
        return;
    }
    // Need to associate inputs with window/camera... we don''t have that for now
    for mut transform in cameras_query.iter_mut() {
        let transform = transform.as_mut();

        if keys.pressed(KeyCode::Z)
            && !keys.any_pressed([
                KeyCode::LShift,
                KeyCode::RShift,
                KeyCode::LControl,
                KeyCode::RControl,
            ])
        {
            camera_options.reset(transform);
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
                camera_translation_change += transform.forward();
            }
            if keys.pressed(KeyCode::S) {
                camera_translation_change -= transform.forward();
            }
            if keys.pressed(KeyCode::A) {
                camera_translation_change -= transform.right();
            }
            if keys.pressed(KeyCode::D) {
                camera_translation_change += transform.right();
            }

            if let Some(gamepad) = gamepad {
                if let Some(left_x) =
                    gamepad_axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
                {
                    camera_translation_change += left_x * transform.right();
                }

                if let Some(left_y) =
                    gamepad_axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
                {
                    camera_translation_change += left_y * transform.forward();
                }
            }

            let mut speed = camera_options.speed.exp();
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

            transform.translation += camera_translation_change;

            let rotation_speed = camera_options.rotation_speed;
            for mouse_motion_event in mouse_motion_events.iter() {
                let rotation = (rotation_speed.degrees() * time.delta_seconds())
                    .min(10.0)
                    .to_radians(); // clamping rotation speed for when it's laggy

                camera_rotation.rotate_yaw_pitch(
                    -mouse_motion_event.delta.x * rotation,
                    -mouse_motion_event.delta.y * rotation,
                );
                transform.rotation = Quat::from_euler(
                    EulerRot::YZX,
                    0.0,
                    camera_rotation.yaw,
                    camera_rotation.pitch,
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
                camera_options.speed = (camera_options.speed + speed_change).clamp(0.0, 10.0);
            }
        }
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

#[cfg(test)]
mod test {
    use lgn_math::{EulerRot, Mat3, Quat, Vec3};

    use crate::{components::camera_component::dir_to_euler, DOWN_VECTOR};

    #[test]
    fn camera() {
        let default_rotation = Quat::from_mat3(&Mat3::IDENTITY);
        assert_eq!(default_rotation, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0));
        assert_eq!(default_rotation.to_euler(EulerRot::YXZ), (0.0, 0.0, 0.0));

        let pitch_90 = Quat::from_mat3(&Mat3::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0,
        ]));
        let (yaw, pitch, roll) = pitch_90.to_euler(EulerRot::YXZ);
        assert!(yaw.abs() < f32::EPSILON);
        assert!((pitch - std::f32::consts::FRAC_PI_2).abs() < 0.003); // It seems like to_euler() is not very precise so not using f32::EPSILON
        assert!(roll.abs() < f32::EPSILON);

        //let eye = Vec3::new(0.0, -2.0, 1.0);
        //let look_at = Vec3::ZERO;
        assert_eq!(dir_to_euler(DOWN_VECTOR), (0.0, 0.0, 0.0));
    }
}
