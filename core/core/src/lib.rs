// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]

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

use legion_app::prelude::*;
use legion_ecs::{
    schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel},
    system::IntoExclusiveSystem,
};

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub enum CoreSystem {
    /// Updates the elapsed time. Any system that interacts with [Time] component should run after
    /// this.
    Time,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Setup the default legion task pools
        app.world
            .get_resource::<DefaultTaskPoolOptions>()
            .cloned()
            .unwrap_or_else(DefaultTaskPoolOptions::default)
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
