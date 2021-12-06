//! Legion Transform
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

pub mod components;
pub mod hierarchy;
pub mod transform_propagate_system;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{components::*, hierarchy::*, TransformPlugin};
}

use lgn_app::prelude::*;
use lgn_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
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
