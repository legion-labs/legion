#![warn(missing_docs)]
//! `lgn_hierarchy` can be used to define hierarchies of entities.
//!
//! Most commonly, these hierarchies are used for inheriting `Transform` values
//! from the [`Parent`] to its [`Children`].

mod components;
pub use components::*;

mod hierarchy;
pub use hierarchy::*;

mod child_builder;
pub use child_builder::*;

mod systems;
pub use systems::*;

#[doc(hidden)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{child_builder::*, components::*, hierarchy::*, HierarchyPlugin};
}

use lgn_app::prelude::*;
use lgn_ecs::prelude::*;

/// The base plugin for handling [`Parent`] and [`Children`] components
#[derive(Default)]
pub struct HierarchyPlugin;

/// Label enum for the systems relating to hierarchy upkeep
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum HierarchySystem {
    /// Updates [`Parent`] when changes in the hierarchy occur
    ParentUpdate,
}

impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            parent_update_system.label(HierarchySystem::ParentUpdate),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            parent_update_system.label(HierarchySystem::ParentUpdate),
        );
    }
}
