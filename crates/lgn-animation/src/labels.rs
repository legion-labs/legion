use lgn_ecs::schedule::StageLabel;

// The names of the animation stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum AnimationStage {
    Update,
}
