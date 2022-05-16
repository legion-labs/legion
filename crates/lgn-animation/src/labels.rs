// use lgn_ecs::schedule::StageLabel;

// /// The names of the physics stages
// #[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
// pub enum AnimationStage {/* */}

use lgn_ecs::schedule::StageLabel;

/// The names of the physics stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum AnimationStage {
    Update,
}
