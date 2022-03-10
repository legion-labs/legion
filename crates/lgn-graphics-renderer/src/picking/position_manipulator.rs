use lgn_ecs::prelude::Commands;
use lgn_math::{Mat4, Quat, Vec2, Vec3};
use lgn_transform::components::Transform;

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
            Mat4::from_axis_angle(Vec3::new(-1.0, 0.0, 0.0), std::f32::consts::PI * 0.5);
        let rotate_z_pointer =
            Mat4::from_axis_angle(Vec3::new(0.0, 0.0, -1.0), std::f32::consts::PI * 0.5);

        let rotate_yz_plane = Mat4::from_axis_angle(Vec3::X, std::f32::consts::PI * 0.5);
        let rotate_xy_plane = Mat4::from_axis_angle(Vec3::Z, std::f32::consts::PI * 0.5);

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
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                1,
                false,
                Transform::from_matrix(rotate_z_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                2,
                false,
                Transform::from_matrix(cone_offset).with_scale(cone_scale),
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                3,
                false,
                Transform::from_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                4,
                false,
                Transform::from_matrix(rotate_x_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshType::Cone,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                5,
                false,
                Transform::from_matrix(rotate_x_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                6,
                true,
                Transform::from_matrix(rotate_yz_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                7,
                true,
                Transform::from_matrix(plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                red,
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

    pub(super) fn manipulate_entity(
        component: AxisComponents,
        base_entity_transform: &Transform,
        camera: &CameraComponent,
        picked_pos: Vec2,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let plane_point = base_entity_transform.translation;
        let plane_normal =
            plane_normal_for_camera_pos(component, base_entity_transform, camera, Quat::IDENTITY);

        let picked_world_point =
            new_world_point_for_cursor(camera, screen_size, picked_pos, plane_point, plane_normal);
        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);

        let delta = new_world_point - picked_world_point;
        let clamped_delta = match component {
            AxisComponents::XAxis => Vec3::new(delta.x, 0.0, 0.0),
            AxisComponents::YAxis => Vec3::new(0.0, delta.y, 0.0),
            AxisComponents::ZAxis => Vec3::new(0.0, 0.0, delta.z),
            _ => delta,
        };

        let new_transform = base_entity_transform;
        new_transform.with_translation(base_entity_transform.translation + clamped_delta)
    }
}
