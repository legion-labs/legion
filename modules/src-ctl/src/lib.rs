// BEGIN - Legion Labs lints v0.1
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::use_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.1
// crate-specific exceptions:
#![allow()]

pub mod attach_branch;
pub mod branch;
pub mod commit;
pub mod config;
pub mod delete;
pub mod detach_branch;
pub mod diff;
pub mod init_local_repository;
pub mod init_workspace;
pub mod local_change;
pub mod lock;
pub mod log;
pub mod merge_branch;
pub mod resolve;
pub mod revert;
pub mod switch_branch;
pub mod sync;
pub mod tree;
pub mod utils;
pub mod workspace;

pub use attach_branch::*;
pub use branch::*;
pub use commit::*;
pub use config::*;
pub use delete::*;
pub use detach_branch::*;
pub use diff::*;
pub use init_local_repository::*;
pub use init_workspace::*;
pub use local_change::*;
pub use lock::*;
pub use log::*;
pub use merge_branch::*;
pub use resolve::*;
pub use revert::*;
pub use switch_branch::*;
pub use sync::*;
pub use tree::*;
pub use utils::*;
pub use workspace::*;
