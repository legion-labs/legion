use lgn_ecs::prelude::{Commands, Entity, Res};
use lgn_graphics_data::Color;
use lgn_input::keyboard::KeyCode;
use lgn_math::{Vec2, Vec3, Vec4, Vec4Swizzles};
use lgn_transform::prelude::Transform;

use crate::{
    components::{CameraComponent, ManipulatorComponent, StaticMesh},
    resources::{DefaultMeshId, DefaultMeshes},
};

use super::{
    PickingIdBlock, PickingManager, PositionComponents, PositionManipulator, RotationComponents,
    RotationManipulator,
};

use std::sync::{Arc, Mutex};

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
        mesh_id: DefaultMeshId,
        commands: &mut Commands<'_, '_>,
        picking_block: &mut PickingIdBlock,
        default_meshes: &DefaultMeshes,
    ) -> Self {
        let mut static_mesh =
            StaticMesh::from_default_meshes(default_meshes, mesh_id as usize, color);

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

pub(super) fn new_world_point_for_cursor(
    camera: &CameraComponent,
    screen_size: Vec2,
    mut cursor_pos: Vec2,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec3 {
    cursor_pos.y = screen_size.y - cursor_pos.y;
    let camera_pos = camera.camera_rig.final_transform.position;
    let ray_point = camera_pos;
    let screen_offset = 2.0 * (cursor_pos / screen_size) - 1.0;

    let (view_matrix, projection_matrix) =
        camera.build_view_projection(screen_size.x as f32, screen_size.y as f32);

    let view_proj_matrix = projection_matrix * view_matrix;
    let inv_view_proj_matrix = view_proj_matrix.inverse();

    let screen_pos = Vec4::new(screen_offset.x, screen_offset.y, 0.1, 1.0);

    let mut world_pos = inv_view_proj_matrix.mul_vec4(screen_pos);
    world_pos = world_pos / world_pos.w;
    let ray_dir = (world_pos.xyz() - camera_pos).normalize();

    intersect_ray_with_plane(ray_point, ray_dir, plane_point, plane_normal)
}

struct ManipulatorManagerInner {
    position: PositionManipulator,
    rotation: RotationManipulator,
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
                rotation: RotationManipulator::new(),
                current_type: ManipulatorType::Rotation,
            })),
        }
    }

    pub fn initialize(
        &mut self,
        mut commands: Commands<'_, '_>,
        default_meshes: Res<'_, DefaultMeshes>,
        picking_manager: Res<'_, PickingManager>,
    ) {
        let mut inner = self.inner.lock().unwrap();

        inner
            .position
            .add_manipulator_parts(&mut commands, &default_meshes, &picking_manager);

        inner
            .rotation
            .add_manipulator_parts(commands, default_meshes, picking_manager);
    }

    pub fn curremt_manipulator_type(&self) -> ManipulatorType {
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
                ManipulatorType::Position => {
                    PositionComponents::from_component_id(selected_part)
                        == PositionComponents::from_component_id(match_part)
                }
                ManipulatorType::Rotation => {
                    RotationComponents::from_component_id(selected_part)
                        == RotationComponents::from_component_id(match_part)
                }
                ManipulatorType::Scale | ManipulatorType::None => panic!(),
            }
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn manipulate_entity(
        &self,
        part: usize,
        base_entity_transform: &Transform,
        camera: &CameraComponent,
        picking_pos_world_space: Vec3,
        screen_size: Vec2,
        cursor_pos: Vec2,
    ) -> Transform {
        let inner = self.inner.lock().unwrap();

        match inner.current_type {
            ManipulatorType::Position => PositionManipulator::manipulate_entity(
                PositionComponents::from_component_id(part),
                base_entity_transform,
                camera,
                picking_pos_world_space,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::Rotation => RotationManipulator::manipulate_entity(
                RotationComponents::from_component_id(part),
                base_entity_transform,
                camera,
                picking_pos_world_space,
                screen_size,
                cursor_pos,
            ),
            ManipulatorType::Scale | ManipulatorType::None => panic!(),
        }
    }

    pub fn change_manipulator(&self, key: KeyCode) {
        let mut inner = self.inner.lock().unwrap();

        inner.current_type = match key {
            KeyCode::Numpad1 | KeyCode::Key1 => ManipulatorType::Position,
            KeyCode::Numpad2 | KeyCode::Key2 => ManipulatorType::Rotation,
            //KeyCode::Numpad3 | KeyCode::Key3 => Some(Key::Num2),
            _ => inner.current_type,
        }
    }
}
