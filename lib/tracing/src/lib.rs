//! Tracing crate
//!
//! Provides logging, metrics, memory and performance profiling
//!
//! Have the lowest impact on the critical path of execution while providing great
//! visibility, `lgn_tracing` focusses on providing predictable performance for hight
//! performance applications. It's primary client is Legion Engine, which runs a
//! distributed, highly compute demanding workloads.
//!
//! Contrary to other tracing crates, lgn-tracing, does not provide hooks for individual
//! events but rather a stream of events, internally it leverages lgn-trancing-transit
//! to serialize the events into a binary format. meant to be consumed later on in process
//! but can also be sent efficiently to over the wire.
//!
//! # Examples
//! ```
//! use lgn_tracing::{
//!    span_scope, info, warn, error, debug, imetric, fmetric, guards, event,
//! };
//!
//! // Initialize tracing, here with a null event sink, see `lgn-telemetry-sink` crate for a proper implementation
//! // libraries don't need (and should not) setup any TelemetrySystemGuard
//! let _telemetry_guard = guards::TelemetrySystemGuard::new(std::sync::Arc::new(event::NullEventSink {}));
//! let _thread_guard = guards::TelemetryThreadGuard::new();
//!
//! // Create a span scope, this will complete when the scope is dropped, and provide the time spent in the scope
//! // Behind the scene this uses a thread local storage
//! // on an i9-11950H this takes around 40ns
//! span_scope!("main");
//!
//! // Logging
//! info!("Hello world");
//! warn!("Hello world");
//! error!("Hello world");
//! debug!("Hello world");
//!
//! // Metrics
//! imetric!("name", "unit", 0);
//! fmetric!("name", "unit", 0.0);
//! ```
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

#[derive(Debug)]
pub struct ProcessInfo {
    pub process_id: String,
    pub exe: String,
    pub username: String,
    pub realname: String,
    pub computer: String,
    pub distro: String,
    pub cpu_brand: String,
    pub tsc_frequency: u64,
    /// RFC 3339
    pub start_time: String,
    pub start_ticks: i64,
    pub parent_process_id: String,
}

pub mod dispatch;
pub mod event;
pub mod guards;
pub mod logs;
pub mod metrics;
pub mod panic_hook;
pub mod spans;

#[macro_use]
mod macros;
mod levels;
mod time;

pub mod prelude {
    pub use crate::levels::*;
    pub use crate::time::*;
    pub use crate::{
        debug, error, fmetric, imetric, info, log, log_enabled, span_scope, trace, warn,
    };
    pub use lgn_tracing_proc_macros::*;
}

pub use prelude::*;
