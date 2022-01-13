use lgn_ecs::schedule::SystemLabel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum CommandBufferLabel {
    Generate,
    Submit,
}
