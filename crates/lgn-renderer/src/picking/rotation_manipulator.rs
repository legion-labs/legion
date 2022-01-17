use lgn_ecs::prelude::Commands;
use lgn_math::{Mat3, Mat4, Quat, Vec2, Vec3};
use lgn_transform::components::Transform;

use crate::{
    components::CameraComponent,
    resources::{DefaultMeshId, DefaultMeshes},
};

use super::{new_world_point_for_cursor, ManipulatorPart, ManipulatorType, PickingManager};

#[derive(Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RotationComponents {
    XAxis = 0,
    YAxis,
    ZAxis,
}

impl RotationComponents {
    pub fn from_component_id(index: usize) -> Self {
        match index {
            0 => Self::XAxis,
            1 => Self::YAxis,
            2 => Self::ZAxis,
            _ => panic!("Unknown index: {}", index),
        }
    }
}

pub(super) struct RotationManipulator {
    parts: Vec<ManipulatorPart>,
}

impl RotationManipulator {
    pub(super) fn new() -> Self {
        Self { parts: Vec::new() }
    }

    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    pub(super) fn add_manipulator_parts(
        &mut self,
        commands: &mut Commands<'_, '_>,
        default_meshes: &DefaultMeshes,
        picking_manager: &PickingManager,
    ) {
        let mut picking_block = picking_manager.acquire_picking_id_block();

        let rotate_x_pointer =
            Mat4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI * 0.5);
        let rotate_y_pointer =
            Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI * 0.5);

        let red = (255, 0, 0).into();
        let green = (0, 255, 0).into();
        let blue = (0, 0, 255).into();

        self.parts = vec![
            ManipulatorPart::new(
                red,
                ManipulatorType::Rotation,
                0,
                false,
                Transform::from_matrix(rotate_x_pointer),
                DefaultMeshId::RotationRing,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Rotation,
                1,
                false,
                Transform::from_matrix(rotate_y_pointer),
                DefaultMeshId::RotationRing,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Rotation,
                2,
                false,
                Transform::from_matrix(Mat4::IDENTITY),
                DefaultMeshId::RotationRing,
                commands,
                &mut picking_block,
                default_meshes,
            ),
        ];

        picking_manager.release_picking_id_block(picking_block);
    }

    pub(super) fn manipulate_entity(
        component: RotationComponents,
        base_entity_transform: &Transform,
        camera: &CameraComponent,
        picked_pos: Vec2,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let plane_point = base_entity_transform.translation;
        let plane_normal = match component {
            RotationComponents::XAxis => Vec3::new(1.0, 0.0, 0.0),
            RotationComponents::YAxis => Vec3::new(0.0, 1.0, 0.0),
            RotationComponents::ZAxis => Vec3::new(0.0, 0.0, 1.0),
        };

        let picked_world_point =
            new_world_point_for_cursor(camera, screen_size, picked_pos, plane_point, plane_normal);
        let dir_to_picked_point =
            (picked_world_point - base_entity_transform.translation).normalize();

        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);
        let dir_to_new_point = (new_world_point - base_entity_transform.translation).normalize();

        let initial_rotation = Mat3::from_quat(base_entity_transform.rotation);
        let new_rotation_angle = dir_to_picked_point.dot(dir_to_new_point).acos();

        let rotation_one = Mat3::from_axis_angle(plane_normal, new_rotation_angle);
        let proj_one = rotation_one
            .mul_vec3(dir_to_picked_point)
            .dot(dir_to_new_point);

        let rotation_two = Mat3::from_axis_angle(plane_normal, -new_rotation_angle);
        let proj_two = rotation_two
            .mul_vec3(dir_to_picked_point)
            .dot(dir_to_new_point);

        let new_rotation = if proj_one > proj_two {
            rotation_one
        } else {
            rotation_two
        } * initial_rotation;

        if !new_rotation.is_nan() {
            base_entity_transform.with_rotation(Quat::from_mat3(&new_rotation))
        } else {
            *base_entity_transform
        }
    }
}
