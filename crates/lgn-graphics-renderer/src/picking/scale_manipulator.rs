use lgn_ecs::prelude::Commands;
use lgn_graphics_data::Color;
use lgn_math::{Mat4, Vec2, Vec3};
use lgn_transform::components::{GlobalTransform, Transform};

use crate::{components::CameraComponent, resources::DefaultMeshType};

use super::{
    new_world_point_for_cursor, plane_normal_for_camera_pos, AxisComponents, ManipulatorPart,
    ManipulatorType, PickingIdContext,
};

pub(super) struct ScaleManipulator {
    parts: Vec<ManipulatorPart>,
}

impl ScaleManipulator {
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
            Mat4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI * 0.5);
        let rotate_y_pointer =
            Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), -std::f32::consts::PI * 0.5);

        let rotate_yz_plane = Mat4::from_axis_angle(Vec3::X, std::f32::consts::PI * 0.5);
        let rotate_xz_plane = Mat4::from_axis_angle(Vec3::Y, -std::f32::consts::PI * 0.5);

        let cube_offset = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.25));
        let plane_offset = Mat4::from_translation(Vec3::new(0.1, 0.1, 0.0));

        let cube_scale = Vec3::new(0.05, 0.05, 0.05);
        let cylinder_scale = Vec3::new(0.0125, 0.0125, 0.25);
        let plane_scale = Vec3::new(0.1, 0.1, 0.1);

        self.parts = vec![
            ManipulatorPart::new(
                Color::RED,
                ManipulatorType::Scale,
                0,
                false,
                Transform::from_matrix(rotate_x_pointer * cube_offset).with_scale(cube_scale),
                DefaultMeshType::Cube,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::RED,
                ManipulatorType::Scale,
                1,
                false,
                Transform::from_matrix(rotate_x_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::GREEN,
                ManipulatorType::Scale,
                2,
                false,
                Transform::from_matrix(rotate_y_pointer * cube_offset).with_scale(cube_scale),
                DefaultMeshType::Cube,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::GREEN,
                ManipulatorType::Scale,
                3,
                false,
                Transform::from_matrix(rotate_y_pointer).with_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::BLUE,
                ManipulatorType::Scale,
                4,
                false,
                Transform::from_matrix(cube_offset).with_scale(cube_scale),
                DefaultMeshType::Cube,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::BLUE,
                ManipulatorType::Scale,
                5,
                false,
                Transform::from_scale(cylinder_scale),
                DefaultMeshType::Cylinder,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::CYAN,
                ManipulatorType::Scale,
                6,
                true,
                Transform::from_matrix(rotate_yz_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::MAGENTA,
                ManipulatorType::Scale,
                7,
                true,
                Transform::from_matrix(plane_offset).with_scale(plane_scale),
                DefaultMeshType::Plane,
                commands,
                picking_context,
            ),
            ManipulatorPart::new(
                Color::ORANGE,
                ManipulatorType::Scale,
                8,
                true,
                Transform::from_matrix(rotate_xz_plane * plane_offset).with_scale(plane_scale),
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
        let entity_rotation = base_global_transform.rotation;
        let inv_entity_rotation = base_global_transform.rotation.inverse();

        let plane_point = base_global_transform.translation;
        let plane_normal =
            plane_normal_for_camera_pos(component, base_global_transform, camera, entity_rotation);

        let picked_world_point =
            new_world_point_for_cursor(camera, screen_size, picked_pos, plane_point, plane_normal);
        let vec_to_picked_point =
            inv_entity_rotation.mul_vec3(picked_world_point - base_global_transform.translation);

        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);
        let vec_to_new_point =
            inv_entity_rotation.mul_vec3(new_world_point - base_global_transform.translation);

        let scale_x = if vec_to_picked_point.x != 0.0 {
            (vec_to_new_point.x / vec_to_picked_point.x).abs()
        } else {
            1.0
        };
        let scale_y = if vec_to_picked_point.y != 0.0 {
            (vec_to_new_point.y / vec_to_picked_point.y).abs()
        } else {
            1.0
        };
        let scale_z = if vec_to_picked_point.z != 0.0 {
            (vec_to_new_point.z / vec_to_picked_point.z).abs()
        } else {
            1.0
        };
        let mut scale_multiplier = Vec3::new(scale_x, scale_y, scale_z);

        scale_multiplier = parent_global_transform.rotation.inverse() * scale_multiplier;
        scale_multiplier /= parent_global_transform.scale;

        let clamped_scale_multiplier = match component {
            AxisComponents::XAxis => Vec3::new(scale_multiplier.x, 1.0, 1.0),
            AxisComponents::YAxis => Vec3::new(1.0, scale_multiplier.y, 1.0),
            AxisComponents::ZAxis => Vec3::new(1.0, 1.0, scale_multiplier.z),
            AxisComponents::XYPlane => Vec3::new(scale_multiplier.x, scale_multiplier.y, 1.0),
            AxisComponents::XZPlane => Vec3::new(scale_multiplier.x, 1.0, scale_multiplier.z),
            AxisComponents::YZPlane => Vec3::new(1.0, scale_multiplier.y, scale_multiplier.z),
        };

        let mut new_transform = *base_local_transform;
        new_transform.scale *= clamped_scale_multiplier;
        new_transform
    }
}
