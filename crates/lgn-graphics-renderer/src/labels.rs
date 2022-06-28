use lgn_ecs::schedule::{StageLabel, SystemLabel};

/// The names of the render stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
    /// TBD
    Prepare,
    /// All work directlly related to command buffer generation
    Render,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum RendererLabel {
    DefaultResourcesInstalled,
    Generate,
}
