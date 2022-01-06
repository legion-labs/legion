use lgn_ecs::prelude::*;
use lgn_transform::components::Transform;

use crate::picking::ManipulatorType;

#[derive(Component)]
pub struct ManipulatorComponent {
    pub part_type: ManipulatorType,
    pub part_num: usize,
    pub local_transform: Transform,
    pub active: bool,
    pub selected: bool,
    pub transparent: bool,
}
