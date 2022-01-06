//! Telemetry Grpc sink library
//!
//! Provides logging, metrics, memory and performance profiling

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

use std::sync::Arc;

use lgn_telemetry::{prelude::*, EventSink};

mod grpc_event_sink;
mod immediate_event_sink;
mod stream;

use grpc_event_sink::GRPCEventSink;
use immediate_event_sink::ImmediateEventSink;

pub type ProcessInfo = lgn_telemetry_proto::telemetry::Process;
pub type StreamInfo = lgn_telemetry_proto::telemetry::Stream;
pub type EncodedBlock = lgn_telemetry_proto::telemetry::Block;
pub use lgn_telemetry_proto::telemetry::ContainerMetadata;

pub struct TelemetryGuard {
    // note we rely here on the drop order being the same as the declaration order
    _thread_guard: TelemetryThreadGuard,
    _guard: TelemetrySystemGuard,
}

impl TelemetryGuard {
    pub fn new() -> anyhow::Result<Self, String> {
        let sink: Arc<dyn EventSink> = match std::env::var("LEGION_TELEMETRY_URL") {
            Ok(url) => Arc::new(GRPCEventSink::new(&url)),
            Err(_no_url_in_env) => Arc::new(ImmediateEventSink::new(
                std::env::var("LGN_TRACE_FILE").ok(),
            )),
        };
        #[cfg(debug_assertions)]
        set_max_log_level(LevelFilter::Info);

        #[cfg(not(debug_assertions))]
        set_max_log_level(LevelFilter::Warn);

        // order here is important
        Ok(Self {
            _guard: TelemetrySystemGuard::new(sink)?,
            _thread_guard: TelemetryThreadGuard::new(),
        })
    }
    pub fn with_log_level(self, leve_filter: LevelFilter) -> Self {
        set_max_log_level(leve_filter);
        self
    }
}
