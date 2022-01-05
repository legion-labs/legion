use lgn_ecs::prelude::{Commands, Entity, Res};
use lgn_graphics_data::Color;
use lgn_math::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use lgn_transform::prelude::Transform;

use crate::{
    components::{ManipulatorComponent, StaticMesh},
    resources::{DefaultMeshId, DefaultMeshes},
};

use super::{PickingIdBlock, PickingManager};

use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
pub enum PositionComponents {
    XAxis = 0,
    YAxis,
    ZAxis,
    XYPlane,
    XZPlane,
    YZPlane,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ManipulatorType {
    Position,
    Rotation,
    Scale,
    None,
}

pub const NUM_POSITION_COMPONENT_PARTS: usize = 9;

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

struct ManipulatorPart {
    _entity: Entity,
}

impl ManipulatorPart {
    #[allow(clippy::too_many_arguments)]
    fn new(
        color: Color,
        part_type: ManipulatorType,
        part_num: usize,
        transparent: bool,
        transform: Transform,
        mesh_id: DefaultMeshId,
        commands: &mut Commands<'_, '_>,
        picking_block: &mut PickingIdBlock,
        default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Self {
        let mut static_mesh =
            StaticMesh::from_default_meshes(default_meshes.as_ref(), mesh_id as usize, color);

        let mut entity_commands = commands.spawn();

        let entity = entity_commands
            .insert(transform)
            .insert(ManipulatorComponent {
                part_type,
                part_num,
                local_translation: transform.translation,
                active: false,
                selected: false,
                transparent,
            })
            .id();

        let picking_id = picking_block.aquire_picking_id(entity).unwrap();
        static_mesh.picking_id = picking_id;

        entity_commands.insert(static_mesh);

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

struct PositionManipulator {
    parts: Vec<ManipulatorPart>,
}

impl PositionManipulator {
    fn new() -> Self {
        Self { parts: Vec::new() }
    }

    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    fn add_manipulator_parts(
        &mut self,
        mut commands: Commands<'_, '_>,
        default_meshes: Res<'_, DefaultMeshes>,
        picking_manager: Res<'_, PickingManager>,
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
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                1,
                false,
                Transform::from_matrix(rotate_z_pointer).with_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                2,
                false,
                Transform::from_matrix(cone_offset).with_scale(cone_scale),
                DefaultMeshId::Cone,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                3,
                false,
                Transform::from_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                4,
                false,
                Transform::from_matrix(rotate_x_pointer * cone_offset).with_scale(cone_scale),
                DefaultMeshId::Cone,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                5,
                false,
                Transform::from_matrix(rotate_x_pointer).with_scale(cylinder_scale),
                DefaultMeshId::Cylinder,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                blue,
                ManipulatorType::Position,
                6,
                true,
                Transform::from_matrix(rotate_xy_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                green,
                ManipulatorType::Position,
                7,
                true,
                Transform::from_matrix(plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
            ManipulatorPart::new(
                red,
                ManipulatorType::Position,
                8,
                true,
                Transform::from_matrix(rotate_yz_plane * plane_offset).with_scale(plane_scale),
                DefaultMeshId::Plane,
                &mut commands,
                &mut picking_block,
                &default_meshes,
            ),
        ];

        picking_manager.release_picking_id_block(picking_block);
    }

    fn manipulate_entity(
        component: PositionComponents,
        base_entity_transform: &Transform,
        camera_transform: &Transform,
        picking_pos_world_space: Vec3,
        fov_y: f32,
        screen_size: Vec2,
        mut cursor_pos: Vec2,
    ) -> Transform {
        let plane_point = base_entity_transform.translation;
        let plane_normal = match component {
            PositionComponents::XAxis | PositionComponents::ZAxis | PositionComponents::XZPlane => {
                Vec3::new(0.0, 1.0, 0.0)
            }
            PositionComponents::YAxis | PositionComponents::XYPlane => Vec3::new(0.0, 0.0, 1.0),
            PositionComponents::YZPlane => Vec3::new(1.0, 0.0, 0.0),
            PositionComponents::None => Vec3::NAN,
        };

        cursor_pos.y = screen_size.y - cursor_pos.y;
        let ray_point = camera_transform.translation;
        let screen_offset = 2.0 * (cursor_pos / screen_size) - 1.0;

        let view_matrix = Mat4::look_at_lh(
            camera_transform.translation,
            camera_transform.translation + camera_transform.forward(),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let aspect_ratio: f32 = screen_size.x / screen_size.y;
        let z_near: f32 = 0.01;
        let z_far: f32 = 100.0;
        let projection_matrix = Mat4::perspective_lh(fov_y, aspect_ratio, z_near, z_far);

        let view_proj_matrix = projection_matrix * view_matrix;
        let inv_view_proj_matrix = view_proj_matrix.inverse();

        let screen_pos = Vec4::new(screen_offset.x, screen_offset.y, 0.1, 1.0);

        let mut world_pos = inv_view_proj_matrix.mul_vec4(screen_pos);
        world_pos = world_pos / world_pos.w;
        let ray_dir = (world_pos.xyz() - camera_transform.translation).normalize();

        let new_world_point =
            intersect_ray_with_plane(ray_point, ray_dir, plane_point, plane_normal);

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

// struct RotationManipulator {}
// struct ScaleManipulator {}

struct ManipulatorManagerInner {
    position: PositionManipulator,
    current_type: ManipulatorType,
}

pub struct ManipulatorManager {
    inner: Arc<Mutex<ManipulatorManagerInner>>,
}

impl ManipulatorManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ManipulatorManagerInner {
                position: PositionManipulator::new(),
                current_type: ManipulatorType::Position,
            })),
        }
    }

    pub fn initialize(
        &mut self,
        commands: Commands<'_, '_>,
        default_meshes: Res<'_, DefaultMeshes>,
        picking_manager: Res<'_, PickingManager>,
    ) {
        let mut inner = self.inner.lock().unwrap();

        inner
            .position
            .add_manipulator_parts(commands, default_meshes, picking_manager);
    }

    pub fn curremt_manipulator_type(&self) -> ManipulatorType {
        let inner = self.inner.lock().unwrap();

        inner.current_type
    }

    #[allow(clippy::too_many_arguments)]
    pub fn manipulate_entity(
        &self,
        component: PositionComponents,
        base_entity_transform: &Transform,
        camera_transform: &Transform,
        picking_pos_world_space: Vec3,
        fov_y: f32,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let inner = self.inner.lock().unwrap();

        match inner.current_type {
            ManipulatorType::Position => PositionManipulator::manipulate_entity(
                component,
                base_entity_transform,
                camera_transform,
                picking_pos_world_space,
                fov_y,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::Rotation | ManipulatorType::Scale | ManipulatorType::None => panic!(),
        }
    }
}
