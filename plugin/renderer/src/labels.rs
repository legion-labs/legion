use lgn_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum RendererSystemLabel {
    FrameUpdate,
}
