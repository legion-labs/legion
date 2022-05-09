use lgn_ecs::prelude::Commands;
use lgn_graphics_data::Color;
use lgn_math::{Mat4, Quat, Vec2, Vec3};
use lgn_transform::components::{GlobalTransform, Transform};

use crate::{components::CameraComponent, resources::DefaultMeshType};

use super::{
    new_world_point_for_cursor, plane_normal_for_camera_pos, AxisComponents, ManipulatorPart,
    ManipulatorType, PickingIdContext,
};

pub(super) struct PositionManipulator {
    parts: Vec<ManipulatorPart>,
}

impl PositionManipulator {
    pub(super) fn new() -> Self {
        Self { parts: Vec::new() }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(super) fn add_manipulator_parts(
        &mut self,
        commands: &mut Commands<'_, '_>,
        picking_context: &mut PickingIdContext<'_>,
    ) {
        let rotate_x_pointer =
            Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), -std::f32::consts::PI * 0.5);
        let rotate_z_pointer =
            Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI * 0.5);

        let rotate_yz_plane = Mat4::from_axis_angle(Vec3::X, -std::f32::consts::PI * 0.5);
        let rotate_xy_plane = Mat4::from_axis_angle(Vec3::Z, std::f32::consts::PI * 0.5);

        let cone_offset = Mat4::from_translation(Vec3::new(0.0, 0.25, 0.0));
        let plane_offset = Mat4::from_translation(Vec3::new(0.1, 0.0, 0.1));

        let cone_scale = Vec3::new(0.05, 0.05, 0.05);
        let cylinder_scale = Vec3::new(0.0125, 0.25, 0.0125);
        let plane_scale = Vec3::new(0.1, 0.1, 0.1);

        self.parts = vec![
            ManipulatorPart::new(
                Color::RED,
                ManipulatorType::Position,
                0,
                false,
                Transform::from_matrix(rotate_x_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::RED,
                ManipulatorType::Position,
                1,
                false,
                Transform::from_matrix(rotate_x_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::GREEN,
                ManipulatorType::Position,
                2,
                false,
                Transform::from_matrix(cone_offset).with_scale(cone_scale),
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::GREEN,
                ManipulatorType::Position,
                3,
                false,
                Transform::from_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::BLUE,
                ManipulatorType::Position,
                4,
                false,
                Transform::from_matrix(rotate_z_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::BLUE,
                ManipulatorType::Position,
                5,
                false,
                Transform::from_matrix(rotate_z_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::CYAN,
                ManipulatorType::Position,
                6,
                true,
                Transform::from_matrix(rotate_yz_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::MAGENTA,
                ManipulatorType::Position,
                7,
                true,
                Transform::from_matrix(plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::YELLOW,
                ManipulatorType::Position,
                8,
                true,
                Transform::from_matrix(rotate_xy_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
        ];
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn manipulate_entity(
        component: AxisComponents,
        base_local_transform: &Transform,
        base_global_transform: &GlobalTransform,
        parent_global_transform: &GlobalTransform,
        camera: &CameraComponent,
        picked_pos: Vec2,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let plane_point = base_global_transform.translation;
        let plane_normal =
            plane_normal_for_camera_pos(component, base_global_transform, camera, Quat::IDENTITY);

        let picked_world_point =
            new_world_point_for_cursor(camera, screen_size, picked_pos, plane_point, plane_normal);
        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);

        let delta = new_world_point - picked_world_point;
        let mut clamped_delta = match component {
            AxisComponents::XAxis => Vec3::new(delta.x, 0.0, 0.0),
            AxisComponents::YAxis => Vec3::new(0.0, delta.y, 0.0),
            AxisComponents::ZAxis => Vec3::new(0.0, 0.0, delta.z),
            _ => delta,
        };

        clamped_delta = parent_global_transform.rotation.inverse() * clamped_delta;
        clamped_delta /= parent_global_transform.scale;

        let new_local_transform = base_local_transform;
        new_local_transform.with_translation(base_local_transform.translation + clamped_delta)
    }
}
