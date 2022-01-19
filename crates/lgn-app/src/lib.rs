//! Legion App
//!
//! This crate is about everything concerning the highest-level, application
//! layer of a Legion app.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
// #![warn(missing_docs)]
//! This crate is about everything concerning the highest-level, application layer of a Bevy
//! app.

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
    pub use crate::{app::App, CoreStage, DynamicPlugin, Plugin, PluginGroup, StartupStage};
}

use lgn_ecs::schedule::StageLabel;

/// The names of the default App stages
///
/// The relative stages are added by [`App::add_default_stages`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    /// Runs only once at the beginning of the app.
    ///
    /// Consists of the sub-stages defined in [`StartupStage`]. Systems added
    /// here are referred to as "startup systems".
    Startup,
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
