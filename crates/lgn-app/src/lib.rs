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
pub use lgn_ecs::event::*;
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

/// The names of the default App stages
///
/// The relative stages are added by [`App::add_default_stages`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    /// Name of app stage that runs before all other app stages
    First,
    /// Name of app stage responsible for performing setup before an update.
    /// Runs before UPDATE.
    PreUpdate,
    /// Name of app stage responsible for doing most app logic. Systems should
    /// be registered here by default.
    Update,
    /// Name of app stage responsible for processing the results of UPDATE. Runs
    /// after UPDATE.
    PostUpdate,
    /// Name of app stage that runs after all other app stages
    Last,
}

/// The label for the Startup [`Schedule`](lgn_ecs::schedule::Schedule),
/// which runs once at the beginning of the app.
///
/// When targeting a [`Stage`](lgn_ecs::schedule::Stage) inside this Schedule,
/// you need to use [`StartupStage`] instead.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct StartupSchedule;

/// The names of the default App startup stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
#[allow(clippy::enum_variant_names)]
pub enum StartupStage {
    /// Name of app stage that runs once before the startup stage
    PreStartup,
    /// Name of app stage that runs once when an app starts up
    Startup,
    /// Name of app stage that runs once after the startup stage
    PostStartup,
}
