//! Telemetry library
//!
//! Provides logging, metrics, memory and performance profiling
//!

// BEGIN - Legion Labs lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
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
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
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
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// END - Legion Labs standard lints v0.4
// crate-specific exceptions:
#![allow(unsafe_code)]

mod compression;
pub mod dispatch;
pub mod dual_time;
pub mod event_block_sink;
pub mod grpc_event_sink;
pub mod guard;
pub mod log_block;
pub mod log_events;
pub mod log_stream;
mod queue_metadata;
pub mod stream;
pub mod thread_stream;

use compression::*;
pub use dispatch::*;
pub use dual_time::*;
pub use event_block_sink::*;
pub use grpc_event_sink::*;
pub use guard::*;
pub use log_block::*;
pub use log_events::*;
pub use log_stream::*;
use queue_metadata::*;
pub use stream::*;
pub use thread_stream::*;

pub use transit::IterableQueue;

pub mod telemetry_ingestion_proto {
    tonic::include_proto!("telemetry_ingestion_proto");
}

pub type ProcessInfo = telemetry_ingestion_proto::Process;
pub type StreamInfo = telemetry_ingestion_proto::Stream;
pub type EncodedBlock = telemetry_ingestion_proto::Block;
pub use telemetry_ingestion_proto::ContainerMetadata;

impl ContainerMetadata {
    pub fn as_transit_udt_vec(&self) -> Vec<transit::UserDefinedType> {
        self.types
            .iter()
            .map(|t| transit::UserDefinedType {
                name: t.name.clone(),
                size: t.size as usize,
                members: t
                    .members
                    .iter()
                    .map(|m| transit::Member {
                        name: m.name.clone(),
                        type_name: m.type_name.clone(),
                        offset: m.offset as usize,
                        size: m.size as usize,
                        is_reference: m.is_reference,
                    })
                    .collect(),
            })
            .collect()
    }
}

impl std::convert::From<&[transit::UserDefinedType]> for ContainerMetadata {
    fn from(src: &[transit::UserDefinedType]) -> Self {
        Self {
            types: src
                .iter()
                .map(|udt| telemetry_ingestion_proto::UserDefinedType {
                    name: udt.name.clone(),
                    size: udt.size as u32,
                    members: udt
                        .members
                        .iter()
                        .map(|member| telemetry_ingestion_proto::UdtMember {
                            name: member.name.clone(),
                            type_name: member.type_name.clone(),
                            offset: member.offset as u32,
                            size: member.size as u32,
                            is_reference: member.is_reference,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
