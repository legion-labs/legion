use lgn_ecs::prelude::*;
use lgn_math::Vec3;

use crate::picking::ManipulatorType;

#[derive(Component)]
pub struct ManipulatorComponent {
    pub part_type: ManipulatorType,
    pub part_num: usize,
    pub local_translation: Vec3,
    pub active: bool,
    pub selected: bool,
    pub transparent: bool,
}
