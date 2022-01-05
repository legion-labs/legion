use lgn_ecs::prelude::Commands;
use lgn_math::{Mat4, Vec2, Vec3};
use lgn_transform::components::Transform;

use crate::{
    components::CameraComponent,
    resources::{DefaultMeshId, DefaultMeshes},
};

use super::{new_world_point_for_cursor, ManipulatorPart, ManipulatorType, PickingManager};

#[derive(Clone, Copy, PartialEq)]
pub enum PositionComponents {
    XAxis = 0,
    YAxis,
    ZAxis,
    XYPlane,
    XZPlane,
    YZPlane,
}

impl PositionComponents {
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

pub(super) struct PositionManipulator {
    parts: Vec<ManipulatorPart>,
}

impl PositionManipulator {
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
        let mut picking_block = picking_manager.aquire_picking_id_block();

        let rotate_x_pointer =
            Mat4::from_axis_angle(Vec3::new(-1.0, 0.0, 0.0), std::f32::consts::PI * 0.5);
        let rotate_z_pointer =
            Mat4::from_axis_angle(Vec3::new(0.0, 0.0, -1.0), std::f32::consts::PI * 0.5);

        let rotate_xy_plane =
            Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI * 0.5);
        let rotate_yz_plane =
            Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::PI * 0.5);

        let cone_offset = Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0));
        let plane_offset = Mat4::from_translation(Vec3::new(0.2, 0.0, -0.2));

        let cone_scale = Vec3::new(0.1, 0.1, 0.1);
        let cylinder_scale = Vec3::new(0.025, 0.5, 0.025);
        let plane_scale = Vec3::new(0.2, 0.2, 0.2);

        let red = (255, 0, 0).into();
        let green = (0, 255, 0).into();
        let blue = (0, 0, 255).into();

        self.parts = vec![
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                0,
                false,
                Transform::from_matrix(rotate_z_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshId::Cone,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                1,
                false,
                Transform::from_matrix(rotate_z_pointer).with_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                2,
                false,
                Transform::from_matrix(cone_offset).with_scale(cone_scale),
                DefaultMeshId::Cone,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                3,
                false,
                Transform::from_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                4,
                false,
                Transform::from_matrix(rotate_x_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshId::Cone,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                5,
                false,
                Transform::from_matrix(rotate_x_pointer).with_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                6,
                true,
                Transform::from_matrix(rotate_xy_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                7,
                true,
                Transform::from_matrix(plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                8,
                true,
                Transform::from_matrix(rotate_yz_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                commands,
                &mut picking_block,
                default_meshes,
            ),
        ];

        picking_manager.release_picking_id_block(picking_block);
    }

    pub(super) fn manipulate_entity(
        component: PositionComponents,
        base_entity_transform: &Transform,
        camera: &CameraComponent,
        picking_pos_world_space: Vec3,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let plane_point = base_entity_transform.translation;
        let plane_normal = match component {
            PositionComponents::XAxis | PositionComponents::ZAxis | PositionComponents::XZPlane => {
                Vec3::new(0.0, 1.0, 0.0)
            }
            PositionComponents::YAxis | PositionComponents::XYPlane => Vec3::new(0.0, 0.0, 1.0),
            PositionComponents::YZPlane => Vec3::new(1.0, 0.0, 0.0),
        };

        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);

        let delta = new_world_point - picking_pos_world_space;
        let clamped_delta = match component {
            PositionComponents::XAxis => Vec3::new(delta.x, 0.0, 0.0),
            PositionComponents::YAxis => Vec3::new(0.0, delta.y, 0.0),
            PositionComponents::ZAxis => Vec3::new(0.0, 0.0, delta.z),
            _ => delta,
        };

        let new_transform = base_entity_transform;
        new_transform.with_translation(base_entity_transform.translation + clamped_delta)
    }
}
