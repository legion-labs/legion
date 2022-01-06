use lgn_ecs::prelude::Commands;
use lgn_math::{Mat4, Vec2, Vec3};
use lgn_transform::components::Transform;

use crate::{
    components::CameraComponent,
    resources::{DefaultMeshId, DefaultMeshes},
};

use super::{
    new_world_point_for_cursor, plane_normal_for_camera_pos, AxisComponents, ManipulatorPart,
    ManipulatorType, PickingManager,
};

pub(super) struct ScaleManipulator {
    parts: Vec<ManipulatorPart>,
}

impl ScaleManipulator {
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

        let cube_offset = Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0));
        let plane_offset = Mat4::from_translation(Vec3::new(0.2, 0.0, -0.2));

        let cube_scale = Vec3::new(0.1, 0.1, 0.1);
        let cylinder_scale = Vec3::new(0.025, 0.5, 0.025);
        let plane_scale = Vec3::new(0.2, 0.2, 0.2);

        let red = (255, 0, 0).into();
        let green = (0, 255, 0).into();
        let blue = (0, 0, 255).into();

        self.parts = vec![
            ManipulatorPart::new(
                red,
                ManipulatorType::Scale,
                0,
                false,
                Transform::from_matrix(rotate_z_pointer * cube_offset).with_scale(cube_scale),
                DefaultMeshId::Cube,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Scale,
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
                ManipulatorType::Scale,
                2,
                false,
                Transform::from_matrix(cube_offset).with_scale(cube_scale),
                DefaultMeshId::Cube,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Scale,
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
                ManipulatorType::Scale,
                4,
                false,
                Transform::from_matrix(rotate_x_pointer * cube_offset).with_scale(cube_scale),
                DefaultMeshId::Cube,
                commands,
                &mut picking_block,
                default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Scale,
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
                ManipulatorType::Scale,
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
                ManipulatorType::Scale,
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
                ManipulatorType::Scale,
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
        component: AxisComponents,
        base_entity_transform: &Transform,
        camera: &CameraComponent,
        picked_pos: Vec2,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let entity_rotation = base_entity_transform.rotation;
        let inv_entity_rotation = base_entity_transform.rotation.inverse();

        let plane_point = base_entity_transform.translation;
        let plane_normal =
            plane_normal_for_camera_pos(component, base_entity_transform, camera, entity_rotation);

        let picked_world_point =
            new_world_point_for_cursor(camera, screen_size, picked_pos, plane_point, plane_normal);
        let vec_to_picked_point =
            inv_entity_rotation.mul_vec3(picked_world_point - base_entity_transform.translation);

        let new_world_point =
            new_world_point_for_cursor(camera, screen_size, cursor_pos, plane_point, plane_normal);
        let vec_to_new_point =
            inv_entity_rotation.mul_vec3(new_world_point - base_entity_transform.translation);

        let scale_multiplier = (vec_to_new_point / vec_to_picked_point).abs();

        let clamped_scale_multiplier = match component {
            AxisComponents::XAxis => Vec3::new(scale_multiplier.x, 1.0, 1.0),
            AxisComponents::YAxis => Vec3::new(1.0, scale_multiplier.y, 1.0),
            AxisComponents::ZAxis => Vec3::new(1.0, 1.0, scale_multiplier.z),
            AxisComponents::XYPlane => Vec3::new(scale_multiplier.x, scale_multiplier.y, 1.0),
            AxisComponents::XZPlane => Vec3::new(scale_multiplier.x, 1.0, scale_multiplier.z),
            AxisComponents::YZPlane => Vec3::new(1.0, scale_multiplier.y, scale_multiplier.z),
        };

        let mut new_transform = *base_entity_transform;
        new_transform.scale *= clamped_scale_multiplier;
        new_transform
    }
}
