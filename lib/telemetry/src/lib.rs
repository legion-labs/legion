//! Telemetry library
//!
//! Provides logging, metrics, memory and performance profiling
//!

// BEGIN - Legion Labs lints v0.5
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]

pub mod compression;
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
pub mod thread_block;
pub mod thread_stream;

pub use compression::*;
pub use dispatch::*;
pub use dual_time::*;
pub use event_block_sink::*;
pub use grpc_event_sink::*;
pub use guard::*;
pub use log_block::*;
pub use log_events::*;
pub use log_stream::*;
use queue_metadata::make_queue_metedata;
pub use stream::*;
pub use thread_block::*;
pub use thread_stream::*;

pub use transit::IterableQueue;

#[allow(clippy::wildcard_imports)]
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
