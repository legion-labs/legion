use lgn_ecs::prelude::*;
use lgn_math::{Quat, Vec3};

use crate::picking::ManipulatorType;

#[derive(Component)]
pub struct ManipulatorComponent {
    pub part_type: ManipulatorType,
    pub part_num: usize,
    pub local_translation: Vec3,
    pub local_rotation: Quat,
    pub active: bool,
    pub selected: bool,
    pub transparent: bool,
}
