//! Legion Core
//!
//! TODO: write documentation.

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
#![allow(clippy::needless_pass_by_value)]

mod bytes;
mod float_ord;
mod label;
mod name;
mod task_pool_options;
mod time;

pub use bytes::*;
pub use float_ord::*;
pub use label::*;
pub use name::*;
pub use task_pool_options::DefaultTaskPoolOptions;
pub use time::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{DefaultTaskPoolOptions, EntityLabels, Labels, Name, Time, Timer};
}

use lgn_app::prelude::*;
use lgn_ecs::{
    schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel},
    system::IntoExclusiveSystem,
};

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub enum CoreSystem {
    /// Updates the elapsed time. Any system that interacts with [Time]
    /// component should run after this.
    Time,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Setup the default legion task pools
        app.world
            .get_resource::<DefaultTaskPoolOptions>()
            .cloned()
            .unwrap_or_default()
            .create_default_pools(&mut app.world);

        app.init_resource::<Time>()
            .init_resource::<EntityLabels>()
            .init_resource::<FixedTimesteps>()
            // time system is added as an "exclusive system" to ensure it runs before other systems
            // in CoreStage::First
            .add_system_to_stage(
                CoreStage::First,
                time_system.exclusive_system().label(CoreSystem::Time),
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, entity_labels_system)
            .add_system_to_stage(CoreStage::PostUpdate, entity_labels_system);
    }
}
