//! Telemetry library
//!
//! Provides logging, metrics, memory and performance profiling
//!

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
#![allow(unsafe_code, clippy::missing_errors_doc)]

pub mod compression;
pub mod dispatch;
pub mod dual_time;
pub mod event_block;
pub mod event_block_sink;
pub mod grpc_event_sink;
pub mod guard;
pub mod log_block;
pub mod log_bridge;
pub mod log_events;
pub mod metric_event;
pub mod metrics_block;
pub mod panic_hook;
mod queue_metadata;
pub mod scope;
pub mod stream;
pub mod thread_block;
pub mod thread_events;

pub use compression::*;
pub use dual_time::*;
pub use event_block_sink::*;
pub use grpc_event_sink::*;
pub use lgn_transit::HeterogeneousQueue;
pub use log_block::*;
pub use log_bridge::*;
pub use metrics_block::*;
pub use scope::*;
pub use stream::*;
pub use thread_block::*;
pub use thread_events::*;

pub type ProcessInfo = lgn_telemetry_proto::telemetry::Process;
pub type StreamInfo = lgn_telemetry_proto::telemetry::Stream;
pub type EncodedBlock = lgn_telemetry_proto::telemetry::Block;
pub use lgn_telemetry_proto::telemetry::ContainerMetadata;

pub mod prelude {
    pub use crate::dispatch::*;
    pub use crate::guard::*;
    pub use crate::log_events::*;
    pub use crate::metric_event::*;
    pub use crate::trace_scope;
}

pub use prelude::*;
