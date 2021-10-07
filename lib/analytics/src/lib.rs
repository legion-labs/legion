//! analytics : provides read access to the telemetry data lake

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
#![allow(clippy::missing_errors_doc)]

use anyhow::{bail, Context, Result};
use prost::Message;
use sqlx::Row;
use std::path::Path;
use telemetry::decompress;
use transit::{parse_object_buffer, read_dependencies, Value};

pub async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_uri = format!("sqlite://{}/telemetry.db3", data_folder.display());
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    Ok(pool)
}

pub async fn fetch_recent_processes(
    connection: &mut sqlx::AnyConnection,
) -> Result<Vec<telemetry::ProcessInfo>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, exe, username, realname, computer, distro, cpu_brand, tsc_frequency, start_time
         FROM processes
         ORDER BY start_time DESC
         LIMIT 100;",
    )
    .fetch_all(connection)
    .await?;
    for r in rows {
        let tsc_frequency: i64 = r.get("tsc_frequency");
        let p = telemetry::ProcessInfo {
            process_id: r.get("process_id"),
            exe: r.get("exe"),
            username: r.get("username"),
            realname: r.get("realname"),
            computer: r.get("computer"),
            distro: r.get("distro"),
            cpu_brand: r.get("cpu_brand"),
            tsc_frequency: tsc_frequency as u64,
            start_time: r.get("start_time"),
        };
        processes.push(p);
    }
    Ok(processes)
}

pub async fn find_process_log_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<telemetry::StreamInfo>> {
    let rows = sqlx::query(
        "SELECT stream_id, process_id, dependencies_metadata, objects_metadata, tags
         FROM streams
         WHERE tags LIKE '%log%'
         AND process_id = ?
         ;",
    )
    .bind(process_id)
    .fetch_all(connection)
    .await
    .with_context(|| "find_process_log_streams")?;
    let mut res = Vec::new();
    for r in rows {
        let stream_id: String = r.get("stream_id");
        let dependencies_metadata_buffer: Vec<u8> = r.get("dependencies_metadata");
        let dependencies_metadata =
            telemetry::telemetry_ingestion_proto::ContainerMetadata::decode(
                &*dependencies_metadata_buffer,
            )
            .with_context(|| "decoding dependencies metadata")?;
        let objects_metadata_buffer: Vec<u8> = r.get("objects_metadata");
        let objects_metadata = telemetry::telemetry_ingestion_proto::ContainerMetadata::decode(
            &*objects_metadata_buffer,
        )
        .with_context(|| "decoding objects metadata")?;
        let tags_str: String = r.get("tags");
        res.push(telemetry::StreamInfo {
            stream_id,
            process_id: r.get("process_id"),
            dependencies_metadata: Some(dependencies_metadata),
            objects_metadata: Some(objects_metadata),
            tags: tags_str.split(' ').map(ToOwned::to_owned).collect(),
        });
    }
    Ok(res)
}

pub async fn find_stream(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<telemetry::StreamInfo> {
    let row = sqlx::query(
        "SELECT process_id, dependencies_metadata, objects_metadata, tags
         FROM streams
         WHERE stream_id = ?
         ;",
    )
    .bind(stream_id)
    .fetch_one(connection)
    .await
    .with_context(|| "find_stream")?;
    let dependencies_metadata_buffer: Vec<u8> = row.get("dependencies_metadata");
    let dependencies_metadata = telemetry::telemetry_ingestion_proto::ContainerMetadata::decode(
        &*dependencies_metadata_buffer,
    )
    .with_context(|| "decoding dependencies metadata")?;
    let objects_metadata_buffer: Vec<u8> = row.get("objects_metadata");
    let objects_metadata =
        telemetry::telemetry_ingestion_proto::ContainerMetadata::decode(&*objects_metadata_buffer)
            .with_context(|| "decoding objects metadata")?;
    let tags_str: String = row.get("tags");
    Ok(telemetry::StreamInfo {
        stream_id: String::from(stream_id),
        process_id: row.get("process_id"),
        dependencies_metadata: Some(dependencies_metadata),
        objects_metadata: Some(objects_metadata),
        tags: tags_str.split(' ').map(ToOwned::to_owned).collect(),
    })
}

pub async fn find_stream_blocks(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<Vec<telemetry::EncodedBlock>> {
    let blocks = sqlx::query(
        "SELECT block_id, begin_time, begin_ticks, end_time, end_ticks
         FROM blocks
         WHERE stream_id = ?;",
    )
    .bind(stream_id)
    .fetch_all(connection)
    .await
    .with_context(|| "find_stream_blocks")?
    .iter()
    .map(|r| {
        let begin_ticks: i64 = r.get("begin_ticks");
        let end_ticks: i64 = r.get("end_ticks");
        telemetry::EncodedBlock {
            block_id: r.get("block_id"),
            stream_id: String::from(stream_id),
            begin_time: r.get("begin_time"),
            begin_ticks: begin_ticks as u64,
            end_time: r.get("end_time"),
            end_ticks: end_ticks as u64,
            payload: None,
        }
    })
    .collect();
    Ok(blocks)
}

pub async fn fetch_block_payload(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    block_id: &str,
) -> Result<telemetry::telemetry_ingestion_proto::BlockPayload> {
    let opt_row = sqlx::query("SELECT payload FROM payloads where block_id = ?;")
        .bind(block_id)
        .fetch_optional(connection)
        .await
        .with_context(|| format!("Fetching payload of block {}", block_id))?;

    let buffer = if let Some(row) = opt_row {
        row.get("payload")
    } else {
        let payload_path = data_path.join("blobs").join(block_id);
        if !payload_path.exists() {
            bail!("payload binary file not found: {}", payload_path.display());
        }
        std::fs::read(&payload_path)
            .with_context(|| format!("reading payload file {}", payload_path.display()))?
    };

    let payload = telemetry::telemetry_ingestion_proto::BlockPayload::decode(&*buffer)
        .with_context(|| format!("reading payload {}", block_id))?;
    Ok(payload)
}

pub fn parse_block<F>(
    stream: &telemetry::StreamInfo,
    payload: &telemetry::telemetry_ingestion_proto::BlockPayload,
    fun: F,
) -> Result<()>
where
    F: FnMut(Value),
{
    let dep_udts = stream
        .dependencies_metadata
        .as_ref()
        .unwrap()
        .as_transit_udt_vec();

    let dependencies = read_dependencies(
        &dep_udts,
        &decompress(&payload.dependencies).with_context(|| "decompressing dependencies payload")?,
    )?;
    let obj_udts = stream
        .objects_metadata
        .as_ref()
        .unwrap()
        .as_transit_udt_vec();
    parse_object_buffer(
        &dependencies,
        &obj_udts,
        &decompress(&payload.objects).with_context(|| "decompressing objects payload")?,
        fun,
    )?;
    Ok(())
}
