use lgn_ecs::schedule::{StageLabel, SystemLabel};

/// The names of the render stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
    /// All work related to resources. REMOVE ASAP
    Resource,
    /// TBD
    Prepare,
    /// All work directlly related to command buffer generation
    Render,
}

//
// TODO(vdbdd): Remove that asap. It should be handled by the resource system.
//
#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum ResourceStageLabel {
    Texture,
    Material,
    Model,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum CommandBufferLabel {
    Generate,
}
