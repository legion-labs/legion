//! Telemetry Ingestion Server
//!
//! Accepts telemetry data throough grpc, stores the metadata in sqlite and the
//! raw event payload in local binary files.
//!
//! Env variables:
//!  - `LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY` : local directory where
//!    data will be dumped

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
//#![allow()]

mod local_ingestion_service;
mod local_telemetry_db;

use std::path::PathBuf;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use lgn_telemetry_proto::ingestion::telemetry_ingestion_server::TelemetryIngestionServer;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::prelude::*;
use local_ingestion_service::connect_to_local_data_lake;
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[clap(name = "Legion Telemetry Ingestion Server")]
#[clap(about = "Legion Telemetry Ingestion Server", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(long, default_value = "[::1]:8080")]
    listen_endpoint: SocketAddr,

    #[clap(subcommand)]
    spec: DataLakeSpec,
}

#[derive(Subcommand, Debug)]
enum DataLakeSpec {
    Local {
        path: PathBuf,
    },

    Remote {
        db_uri: String,
        username: String,
        password: String,
        bucket_uri: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::default().unwrap();
    let args = Cli::parse();
    let service = match args.spec {
        DataLakeSpec::Local { path } => connect_to_local_data_lake(path).await?,
        DataLakeSpec::Remote {
            db_uri: _,
            username: _,
            password: _,
            bucket_uri: _,
        } => {
            panic!("remote");
        }
    };
    Server::builder()
        .add_service(TelemetryIngestionServer::new(service))
        .serve(args.listen_endpoint)
        .await?;
    Ok(())
}
