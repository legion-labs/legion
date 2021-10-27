//! analytics : provides read access to the telemetry data lake

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
#![allow(clippy::missing_errors_doc)]

use anyhow::{bail, Context, Result};
use legion_telemetry::{decompress, ContainerMetadata};
use prost::Message;
use sqlx::Row;
use std::path::Path;
use transit::{parse_object_buffer, read_dependencies, Member, UserDefinedType, Value};

pub async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_uri = format!("sqlite://{}/telemetry.db3", data_folder.display());
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    Ok(pool)
}

fn process_from_row(row: &sqlx::any::AnyRow) -> legion_telemetry::ProcessInfo {
    let tsc_frequency: i64 = row.get("tsc_frequency");
    let start_ticks: i64 = row.get("start_ticks");
    legion_telemetry::ProcessInfo {
        process_id: row.get("process_id"),
        exe: row.get("exe"),
        username: row.get("username"),
        realname: row.get("realname"),
        computer: row.get("computer"),
        distro: row.get("distro"),
        cpu_brand: row.get("cpu_brand"),
        tsc_frequency: tsc_frequency as u64,
        start_time: row.get("start_time"),
        start_ticks: start_ticks as u64,
        parent_process_id: row.get("parent_process_id"),
    }
}

pub async fn processes_by_name_substring(
    connection: &mut sqlx::AnyConnection,
    filter: &str,
) -> Result<Vec<legion_telemetry::ProcessInfo>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, exe, username, realname, computer, distro, cpu_brand, tsc_frequency, start_time, start_ticks, parent_process_id
         FROM processes
         WHERE exe LIKE ?
         ORDER BY start_time DESC
         LIMIT 100;",
    )
    .bind( format!("%{}%", filter) )
    .fetch_all(connection)
    .await?;
    for r in rows {
        processes.push(process_from_row(&r));
    }
    Ok(processes)
}

pub async fn find_process(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<legion_telemetry::ProcessInfo> {
    let row = sqlx::query(
        "SELECT process_id, exe, username, realname, computer, distro, cpu_brand, tsc_frequency, start_time, start_ticks, parent_process_id
         FROM processes
         WHERE process_id = ?;",
    )
    .bind(process_id)
    .fetch_one(connection)
    .await?;
    Ok(process_from_row(&row))
}

pub async fn fetch_recent_processes(
    connection: &mut sqlx::AnyConnection,
) -> Result<Vec<legion_telemetry::ProcessInfo>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, exe, username, realname, computer, distro, cpu_brand, tsc_frequency, start_time, start_ticks, parent_process_id
         FROM processes
         ORDER BY start_time DESC
         LIMIT 100;",
    )
    .fetch_all(connection)
    .await?;
    for r in rows {
        processes.push(process_from_row(&r));
    }
    Ok(processes)
}

pub async fn fetch_child_processes(
    connection: &mut sqlx::AnyConnection,
    parent_process_id: &str,
) -> Result<Vec<legion_telemetry::ProcessInfo>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, exe, username, realname, computer, distro, cpu_brand, tsc_frequency, start_time, start_ticks, parent_process_id
         FROM processes
         WHERE parent_process_id = ?
         ORDER BY start_time DESC
         ;",
    )
    .bind(parent_process_id)
    .fetch_all(connection)
    .await?;
    for r in rows {
        processes.push(process_from_row(&r));
    }
    Ok(processes)
}

pub async fn find_process_streams_tagged(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
    tag: &str,
) -> Result<Vec<legion_telemetry::StreamInfo>> {
    let rows = sqlx::query(&format!(
        "SELECT stream_id, process_id, dependencies_metadata, objects_metadata, tags, properties
         FROM streams
         WHERE tags LIKE '%{}%'
         AND process_id = ?
         ;",
        tag
    ))
    .bind(process_id)
    .fetch_all(connection)
    .await
    .with_context(|| "fetch_all in find_process_streams_tagged")?;
    let mut res = Vec::new();
    for r in rows {
        let stream_id: String = r.get("stream_id");
        let dependencies_metadata_buffer: Vec<u8> = r.get("dependencies_metadata");
        let dependencies_metadata = legion_telemetry_proto::ingestion::ContainerMetadata::decode(
            &*dependencies_metadata_buffer,
        )
        .with_context(|| "decoding dependencies metadata")?;
        let objects_metadata_buffer: Vec<u8> = r.get("objects_metadata");
        let objects_metadata =
            legion_telemetry_proto::ingestion::ContainerMetadata::decode(&*objects_metadata_buffer)
                .with_context(|| "decoding objects metadata")?;
        let tags_str: String = r.get("tags");
        let properties_str: String = r.get("properties");
        let properties: std::collections::HashMap<String, String> =
            serde_json::from_str(&properties_str).unwrap();
        res.push(legion_telemetry::StreamInfo {
            stream_id,
            process_id: r.get("process_id"),
            dependencies_metadata: Some(dependencies_metadata),
            objects_metadata: Some(objects_metadata),
            tags: tags_str.split(' ').map(ToOwned::to_owned).collect(),
            properties,
        });
    }
    Ok(res)
}

pub async fn find_process_log_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<legion_telemetry::StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "log").await
}

pub async fn find_process_thread_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<legion_telemetry::StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "cpu").await
}

pub async fn find_process_metrics_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<legion_telemetry::StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "metrics").await
}

pub async fn find_stream(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<legion_telemetry::StreamInfo> {
    let row = sqlx::query(
        "SELECT process_id, dependencies_metadata, objects_metadata, tags, properties
         FROM streams
         WHERE stream_id = ?
         ;",
    )
    .bind(stream_id)
    .fetch_one(connection)
    .await
    .with_context(|| "find_stream")?;
    let dependencies_metadata_buffer: Vec<u8> = row.get("dependencies_metadata");
    let dependencies_metadata = legion_telemetry_proto::ingestion::ContainerMetadata::decode(
        &*dependencies_metadata_buffer,
    )
    .with_context(|| "decoding dependencies metadata")?;
    let objects_metadata_buffer: Vec<u8> = row.get("objects_metadata");
    let objects_metadata =
        legion_telemetry_proto::ingestion::ContainerMetadata::decode(&*objects_metadata_buffer)
            .with_context(|| "decoding objects metadata")?;
    let tags_str: String = row.get("tags");
    let properties_str: String = row.get("properties");
    let properties: std::collections::HashMap<String, String> =
        serde_json::from_str(&properties_str).unwrap();
    Ok(legion_telemetry::StreamInfo {
        stream_id: String::from(stream_id),
        process_id: row.get("process_id"),
        dependencies_metadata: Some(dependencies_metadata),
        objects_metadata: Some(objects_metadata),
        tags: tags_str.split(' ').map(ToOwned::to_owned).collect(),
        properties,
    })
}

pub async fn find_stream_blocks(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<Vec<legion_telemetry::EncodedBlock>> {
    let blocks = sqlx::query(
        "SELECT block_id, begin_time, begin_ticks, end_time, end_ticks
         FROM blocks
         WHERE stream_id = ?
         ORDER BY begin_time;",
    )
    .bind(stream_id)
    .fetch_all(connection)
    .await
    .with_context(|| "find_stream_blocks")?
    .iter()
    .map(|r| {
        let begin_ticks: i64 = r.get("begin_ticks");
        let end_ticks: i64 = r.get("end_ticks");
        legion_telemetry::EncodedBlock {
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
) -> Result<legion_telemetry_proto::ingestion::BlockPayload> {
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

    let payload = legion_telemetry_proto::ingestion::BlockPayload::decode(&*buffer)
        .with_context(|| format!("reading payload {}", block_id))?;
    Ok(payload)
}

fn container_metadata_as_transit_udt_vec(
    value: &ContainerMetadata,
) -> Vec<transit::UserDefinedType> {
    value
        .types
        .iter()
        .map(|t| UserDefinedType {
            name: t.name.clone(),
            size: t.size as usize,
            members: t
                .members
                .iter()
                .map(|m| Member {
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

// parse_block calls fun for each object in the block until fun returns `false`
pub fn parse_block<F>(
    stream: &legion_telemetry::StreamInfo,
    payload: &legion_telemetry_proto::ingestion::BlockPayload,
    fun: F,
) -> Result<()>
where
    F: FnMut(Value) -> bool,
{
    let dep_udts =
        container_metadata_as_transit_udt_vec(stream.dependencies_metadata.as_ref().unwrap());
    let dependencies = read_dependencies(
        &dep_udts,
        &decompress(&payload.dependencies).with_context(|| "decompressing dependencies payload")?,
    )?;
    let obj_udts = container_metadata_as_transit_udt_vec(stream.objects_metadata.as_ref().unwrap());
    parse_object_buffer(
        &dependencies,
        &obj_udts,
        &decompress(&payload.objects).with_context(|| "decompressing objects payload")?,
        fun,
    )?;
    Ok(())
}

fn format_log_level(level: u8) -> &'static str {
    match level {
        1 => "Info",
        2 => "Warning",
        3 => "Error",
        _ => "Unknown",
    }
}

// find_process_log_entry calls pred(time_ticks,entry_str) with each log entry until pred returns Some(x)
pub async fn find_process_log_entry<Res, Predicate: FnMut(u64, String) -> Option<Res>>(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
    mut pred: Predicate,
) -> Result<Option<Res>> {
    let mut found_entry = None;
    for stream in find_process_log_streams(connection, process_id).await? {
        for b in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload = fetch_block_payload(connection, data_path, &b.block_id).await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    match obj.type_name.as_str() {
                        "LogMsgEvent" | "LogDynMsgEvent" => {
                            let time = obj.get::<u64>("time").unwrap();
                            let entry = format!(
                                "[{}] {}",
                                format_log_level(obj.get::<u8>("level").unwrap()),
                                obj.get::<String>("msg").unwrap()
                            );
                            if let Some(x) = pred(time, entry) {
                                found_entry = Some(x);
                                return false; //do not continue
                            }
                        }
                        _ => {}
                    }
                }
                true //continue
            })?;
            if found_entry.is_some() {
                return Ok(found_entry);
            }
        }
    }
    Ok(found_entry)
}

pub async fn for_each_process_log_entry<ProcessLogEntry: FnMut(u64, String)>(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
    mut process_log_entry: ProcessLogEntry,
) -> Result<()> {
    find_process_log_entry(connection, data_path, process_id, |time, entry| {
        process_log_entry(time, entry);
        let nothing: Option<()> = None;
        nothing //continue searching
    })
    .await?;
    Ok(())
}

pub async fn for_each_process_metric<ProcessMetric: FnMut(transit::Object)>(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
    mut process_metric: ProcessMetric,
) -> Result<()> {
    for stream in find_process_metrics_streams(connection, process_id).await? {
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload = fetch_block_payload(connection, data_path, &block.block_id).await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    process_metric(obj);
                }
                true //continue
            })?;
        }
    }
    Ok(())
}

#[async_recursion::async_recursion]
pub async fn for_each_process_in_tree<F>(
    pool: &sqlx::AnyPool,
    root: &legion_telemetry::ProcessInfo,
    rec_level: u16,
    fun: F,
) -> Result<()>
where
    F: Fn(&legion_telemetry::ProcessInfo, u16) + std::marker::Send + Clone,
{
    fun(root, rec_level);
    let mut connection = pool.acquire().await?;
    for child_info in fetch_child_processes(&mut connection, &root.process_id)
        .await
        .unwrap()
    {
        let fun_clone = fun.clone();
        for_each_process_in_tree(pool, &child_info, rec_level + 1, fun_clone).await?;
    }
    Ok(())
}

pub mod prelude {
    pub use crate::alloc_sql_pool;
    pub use crate::fetch_block_payload;
    pub use crate::fetch_child_processes;
    pub use crate::fetch_recent_processes;
    pub use crate::find_process;
    pub use crate::find_process_log_entry;
    pub use crate::find_process_thread_streams;
    pub use crate::find_stream_blocks;
    pub use crate::for_each_process_in_tree;
    pub use crate::for_each_process_log_entry;
    pub use crate::for_each_process_metric;
    pub use crate::parse_block;
    pub use crate::processes_by_name_substring;
}
