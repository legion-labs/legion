use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_transform::components::Transform;

use crate::{
    picking::{ManipulatorType, PickingId},
    resources::DefaultMeshType,
};

#[derive(Component)]
pub struct ManipulatorComponent {
    pub part_type: ManipulatorType,
    pub part_num: usize,
    pub local_transform: Transform,
    pub active: bool,
    pub selected: bool,
    pub transparent: bool,
    pub picking_id: PickingId,
    pub default_mesh_type: DefaultMeshType,
    pub color: Color,
}
