//! Legion App
//!
//! This crate is about everything concerning the highest-level, application
//! layer of a Legion app.

// crate-specific lint exceptions:
// #![warn(missing_docs)]

mod app;
mod plugin;
mod plugin_group;
mod schedule_runner;

#[cfg(feature = "lgn_ci_testing")]
mod ci_testing;

pub use app::*;
pub use lgn_derive::DynamicPlugin;
pub use plugin::*;
pub use plugin_group::*;
pub use schedule_runner::*;

#[allow(missing_docs)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        app::App, CoreStage, DynamicPlugin, Plugin, PluginGroup, StartupSchedule, StartupStage,
    };
}

use lgn_ecs::schedule::StageLabel;

/// The names of the default [`App`] stages.
///
/// The relative [`Stages`](lgn_ecs::schedule::Stage) are added by [`App::add_default_stages`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs before all other app stages.
    First,
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs before [`CoreStage::Update`].
    PreUpdate,
    /// The [`Stage`](lgn_ecs::schedule::Stage) responsible for doing most app logic. Systems should be registered here by default.
    Update,
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs after [`CoreStage::Update`].
    PostUpdate,
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs after all other app stages.
    Last,
}

/// The label for the startup [`Schedule`](lgn_ecs::schedule::Schedule),
/// which runs once at the beginning of the [`App`].
///
/// When targeting a [`Stage`](lgn_ecs::schedule::Stage) inside this [`Schedule`](lgn_ecs::schedule::Schedule),
/// you need to use [`StartupStage`] instead.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct StartupSchedule;

/// The names of the default [`App`] startup stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
#[allow(clippy::enum_variant_names)]
pub enum StartupStage {
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs once before [`StartupStage::Startup`].
    PreStartup,
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs once when an [`App`] starts up.
    Startup,
    /// The [`Stage`](lgn_ecs::schedule::Stage) that runs once after [`StartupStage::Startup`].
    PostStartup,
}
