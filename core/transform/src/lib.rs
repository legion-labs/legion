pub mod components;
pub mod hierarchy;
pub mod transform_propagate_system;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{components::*, hierarchy::*, TransformPlugin};
}

use legion_app::prelude::*;
use legion_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
use prelude::parent_update_system;

#[derive(Default)]
pub struct TransformPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum TransformSystem {
    TransformPropagate,
    ParentUpdate,
}

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app
            // add transform systems to startup so the first update is "correct"
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                parent_update_system.label(TransformSystem::ParentUpdate),
            )
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                transform_propagate_system::transform_propagate_system
                    .label(TransformSystem::TransformPropagate)
                    .after(TransformSystem::ParentUpdate),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                parent_update_system.label(TransformSystem::ParentUpdate),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                transform_propagate_system::transform_propagate_system
                    .label(TransformSystem::TransformPropagate)
                    .after(TransformSystem::ParentUpdate),
            );
    }
}
