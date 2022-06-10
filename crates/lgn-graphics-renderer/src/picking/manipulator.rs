use lgn_ecs::prelude::{Commands, Entity, Res};
use lgn_graphics_data::Color;
use lgn_input::keyboard::KeyCode;
use lgn_math::{Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use lgn_transform::prelude::{GlobalTransform, Transform};

use crate::{
    components::{CameraComponent, ManipulatorComponent},
    resources::DefaultMeshType,
};

use super::{
    position_manipulator::PositionManipulator,
    rotation_manipulator::{RotationComponents, RotationManipulator},
    scale_manipulator::ScaleManipulator,
    PickingIdContext, PickingManager,
};

use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
pub enum AxisComponents {
    XAxis = 0,
    YAxis,
    ZAxis,
    XYPlane,
    XZPlane,
    YZPlane,
}

impl AxisComponents {
    pub fn from_component_id(index: usize) -> Self {
        match index {
            0 | 1 => Self::XAxis,
            2 | 3 => Self::YAxis,
            4 | 5 => Self::ZAxis,
            6 => Self::XYPlane,
            7 => Self::XZPlane,
            8 => Self::YZPlane,
            _ => panic!("Unknown index: {}", index),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ManipulatorType {
    Position,
    Rotation,
    Scale,
    None,
}

pub(super) struct ManipulatorPart {
    _entity: Entity,
}

impl ManipulatorPart {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        color: Color,
        part_type: ManipulatorType,
        part_num: usize,
        transparent: bool,
        transform: Transform,
        default_mesh_type: DefaultMeshType,
        commands: &mut Commands<'_, '_>,
        picking_context: &mut PickingIdContext<'_>,
    ) -> Self {
        let mut entity_commands = commands.spawn();
        let entity = entity_commands
            .insert(transform)
            .insert(GlobalTransform::identity())
            .id();

        entity_commands.insert(ManipulatorComponent {
            part_type,
            part_num,
            local_transform: transform,
            active: false,
            selected: false,
            transparent,
            picking_id: picking_context.acquire_picking_id(entity),
            default_mesh_type,
            color,
        });

        Self { _entity: entity }
    }
}

fn intersect_ray_with_plane(
    ray_point: Vec3,
    ray_dir: Vec3,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec3 {
    // arbitrary value
    let too_small = 0.0001;

    let diff = ray_point - plane_point;
    let prod_1 = diff.dot(plane_normal);
    let prod_2 = ray_dir.dot(plane_normal);

    if prod_2.abs() > too_small {
        let prod_3 = prod_1 / prod_2;
        ray_point - ray_dir * prod_3
    } else {
        Vec3::new(f32::MAX, f32::MAX, f32::MAX)
    }
}

pub(super) fn new_world_point_for_cursor(
    camera: &CameraComponent,
    screen_size: Vec2,
    mut cursor_pos: Vec2,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec3 {
    cursor_pos.y = screen_size.y - cursor_pos.y;
    let camera_pos = camera.position();
    let ray_point = camera_pos;
    let screen_offset = 2.0 * (cursor_pos / screen_size) - 1.0;
    let screen_pos = Vec4::new(screen_offset.x, screen_offset.y, 0.5, 1.0);

    let projection_matrix = camera.build_projection(screen_size.x as f32, screen_size.y as f32);
    let mut view_pos = projection_matrix.inverse().mul_vec4(screen_pos);
    view_pos /= view_pos.w;

    let view_matrix = camera.view_transform().compute_matrix();
    let world_pos = view_matrix.inverse().mul_vec4(view_pos);

    let ray_dir = (world_pos.xyz() - camera_pos).normalize();

    intersect_ray_with_plane(ray_point, ray_dir, plane_point, plane_normal)
}

pub(super) fn plane_normal_for_camera_pos(
    component: AxisComponents,
    base_entity_transform: &GlobalTransform,
    camera: &CameraComponent,
    rotation: Quat,
) -> Vec3 {
    let camera_pos = camera.position();
    let dir_to_camera = (camera_pos - base_entity_transform.translation).normalize();

    let xy_plane_normal = rotation.mul_vec3(Vec3::Z);
    let zx_plane_normal = rotation.mul_vec3(Vec3::Y);
    let yz_plane_normal = rotation.mul_vec3(Vec3::X);

    let xy_plane_facing_cam = dir_to_camera.dot(xy_plane_normal).abs();
    let xz_plane_facing_cam = dir_to_camera.dot(zx_plane_normal).abs();
    let yz_plane_facing_cam = dir_to_camera.dot(yz_plane_normal).abs();

    match component {
        AxisComponents::XAxis => {
            if xy_plane_facing_cam > xz_plane_facing_cam {
                xy_plane_normal
            } else {
                zx_plane_normal
            }
        }
        AxisComponents::YAxis => {
            if xy_plane_facing_cam > yz_plane_facing_cam {
                xy_plane_normal
            } else {
                yz_plane_normal
            }
        }
        AxisComponents::ZAxis => {
            if xz_plane_facing_cam > yz_plane_facing_cam {
                zx_plane_normal
            } else {
                yz_plane_normal
            }
        }
        AxisComponents::XYPlane => xy_plane_normal,
        AxisComponents::XZPlane => zx_plane_normal,
        AxisComponents::YZPlane => yz_plane_normal,
    }
}

struct ManipulatorManagerInner {
    position: PositionManipulator,
    rotation: RotationManipulator,
    scale: ScaleManipulator,
    current_type: ManipulatorType,
}

#[derive(Clone)]
pub struct ManipulatorManager {
    inner: Arc<Mutex<ManipulatorManagerInner>>,
}

impl ManipulatorManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ManipulatorManagerInner {
                position: PositionManipulator::new(),
                rotation: RotationManipulator::new(),
                scale: ScaleManipulator::new(),
                current_type: ManipulatorType::Position,
            })),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn initialize(
        &mut self,
        mut commands: Commands<'_, '_>,
        picking_manager: Res<'_, PickingManager>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let mut picking_context = PickingIdContext::new(&picking_manager);

        inner
            .position
            .add_manipulator_parts(&mut commands, &mut picking_context);

        inner
            .rotation
            .add_manipulator_parts(&mut commands, &mut picking_context);

        inner
            .scale
            .add_manipulator_parts(&mut commands, &mut picking_context);
    }

    pub fn current_manipulator_type(&self) -> ManipulatorType {
        let inner = self.inner.lock().unwrap();

        inner.current_type
    }

    pub fn match_manipulator_parts(
        &self,
        selected_part: usize,
        match_type: ManipulatorType,
        match_part: usize,
    ) -> bool {
        let inner = self.inner.lock().unwrap();

        if inner.current_type == match_type {
            match inner.current_type {
                ManipulatorType::Position | ManipulatorType::Scale => {
                    AxisComponents::from_component_id(selected_part)
                        == AxisComponents::from_component_id(match_part)
                }
                ManipulatorType::Rotation => {
                    RotationComponents::from_component_id(selected_part)
                        == RotationComponents::from_component_id(match_part)
                }
                ManipulatorType::None => panic!(),
            }
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn manipulate_entity(
        &self,
        part: usize,
        base_local_transform: &Transform,
        base_global_transform: &GlobalTransform,
        parent_global_transform: &GlobalTransform,
        camera: &CameraComponent,
        picked_pos: Vec2,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let inner = self.inner.lock().unwrap();

        match inner.current_type {
            ManipulatorType::Position => PositionManipulator::manipulate_entity(
                AxisComponents::from_component_id(part),
                base_local_transform,
                base_global_transform,
                parent_global_transform,
                camera,
                picked_pos,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::Rotation => RotationManipulator::manipulate_entity(
                RotationComponents::from_component_id(part),
                base_local_transform,
                base_global_transform,
                parent_global_transform,
                camera,
                picked_pos,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::Scale => ScaleManipulator::manipulate_entity(
                AxisComponents::from_component_id(part),
                base_local_transform,
                base_global_transform,
                parent_global_transform,
                camera,
                picked_pos,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::None => panic!(),
        }
    }

    pub fn change_manipulator(&self, key: KeyCode) {
        let mut inner = self.inner.lock().unwrap();

        inner.current_type = match key {
            KeyCode::Numpad1 | KeyCode::Key1 => ManipulatorType::Position,
            KeyCode::Numpad2 | KeyCode::Key2 => ManipulatorType::Rotation,
            KeyCode::Numpad3 | KeyCode::Key3 => ManipulatorType::Scale,
            _ => inner.current_type,
        }
    }

    pub fn manipulator_transform_from_entity_transform(
        &self,
        entity_transform: &GlobalTransform,
        manipulator_transform: &mut Transform,
    ) {
        let inner = self.inner.lock().unwrap();

        *manipulator_transform = Transform::from_translation(entity_transform.translation);
        if inner.current_type == ManipulatorType::Scale {
            *manipulator_transform = manipulator_transform.with_rotation(entity_transform.rotation);
        }
    }

    pub fn scale_manipulator_for_viewport(
        entity_transform: &GlobalTransform,
        manipulator_transform: &Transform,
        projection_matrix: Mat4,
        camera_transform: &GlobalTransform,
    ) -> GlobalTransform {
        let view_pos = camera_transform.mul_vec3(entity_transform.translation);
        let x_offset = view_pos + Vec3::new(0.5, 0.0, 0.0);
        let y_offset = view_pos + Vec3::new(0.0, 0.5, 0.0);

        let proj_pos = projection_matrix.mul_vec4(view_pos.extend(1.0));
        let x_proj = projection_matrix.mul_vec4(x_offset.extend(1.0));
        let y_proj = projection_matrix.mul_vec4(y_offset.extend(1.0));

        let x_scale = 0.2 / ((x_proj.x / x_proj.w) - (proj_pos.x / proj_pos.w));
        let y_scale = 0.2 / ((y_proj.y / y_proj.w) - (proj_pos.y / proj_pos.w));

        let manip_scale = x_scale + y_scale * 0.5;

        let scale_transform =
            Transform::from_scale(Vec3::new(manip_scale, manip_scale, manip_scale));

        entity_transform.mul_transform(scale_transform.mul_transform(*manipulator_transform))
    }
}
