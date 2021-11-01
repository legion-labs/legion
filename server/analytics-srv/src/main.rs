//! Legion Analytics Server
//!
//! Feeds data to the analytics-web interface.
//!
//! Env variables:
//!  - `LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY` : local telemetry directory
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
#![allow()]

mod analytics_service;

use analytics_service::AnalyticsService;
use anyhow::{Context, Result};
use legion_analytics::alloc_sql_pool;
use legion_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalyticsServer;
use std::path::PathBuf;
use tonic::transport::Server;

fn get_data_directory() -> Result<PathBuf> {
    let folder =
        std::env::var("LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY").with_context(|| {
            String::from("Error reading env variable LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY")
        })?;
    Ok(PathBuf::from(folder))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init().unwrap();
    let addr = "127.0.0.1:9090".parse()?;
    let data_dir = get_data_directory()?;
    let pool = alloc_sql_pool(&data_dir).await?;
    let service = AnalyticsService::new(pool, data_dir);
    log::info!("service allocated");
    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(PerformanceAnalyticsServer::new(service)))
        .serve(addr)
        .await?;
    Ok(())
}
