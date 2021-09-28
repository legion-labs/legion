//! Telemetry Ingestion Server
//!
//! Accepts telemetry data throough grpc, stores the metadata in sqlite and the raw event payload in local binary files.
//!
//! Env variables:
//!  - `LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY` : local directory where data will be dumped
//!

// BEGIN - Legion Labs lints v0.3
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
    rust_2018_idioms,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
// END - Legion Labs standard lints v0.3
// crate-specific exceptions:
#![allow()]

mod local_ingestion_service;
use local_ingestion_service::*;

use anyhow::{Context, Result};
use sqlx::migrate::MigrateDatabase;
use std::path::{Path, PathBuf};
use telemetry::telemetry_ingestion_proto::telemetry_ingestion_server::TelemetryIngestionServer;
use tonic::transport::Server;

fn get_data_directory() -> Result<PathBuf> {
    let folder =
        std::env::var("LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY").with_context(|| {
            String::from("Error reading env variable LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY")
        })?;
    Ok(PathBuf::from(folder))
}

async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_path = data_folder.join("telemetry.db3");
    let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    if !sqlx::Any::database_exists(&db_uri)
        .await
        .with_context(|| String::from("Searching for telemetry database"))?
    {
        sqlx::Any::create_database(&db_uri)
            .await
            .with_context(|| String::from("Creating telemetry database"))?;
    }
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse()?;

    let data_folder = get_data_directory()?;
    if !data_folder.exists() {
        std::fs::create_dir_all(&data_folder)
            .with_context(|| format!("Error creating data folder {}", data_folder.display()))?;
    }

    let db_pool = alloc_sql_pool(&data_folder).await?;
    let service = LocalIngestionService::new(db_pool);

    Server::builder()
        .add_service(TelemetryIngestionServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
