//! analytics : provides read access to the telemetry data lake

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::decompress;
use lgn_telemetry_proto::telemetry::{
    Block as EncodedBlock, ContainerMetadata, Process as ProcessInfo, Stream as StreamInfo,
};
use lgn_tracing::prelude::*;
use lgn_tracing_transit::{parse_object_buffer, read_dependencies, Member, UserDefinedType, Value};
use prost::Message;
use sqlx::Row;

pub async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_uri = format!("sqlite://{}/telemetry.db3", data_folder.display());
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    Ok(pool)
}

fn process_from_row(row: &sqlx::any::AnyRow) -> ProcessInfo {
    let tsc_frequency: i64 = row.get("tsc_frequency");
    ProcessInfo {
        process_id: row.get("process_id"),
        exe: row.get("exe"),
        username: row.get("username"),
        realname: row.get("realname"),
        computer: row.get("computer"),
        distro: row.get("distro"),
        cpu_brand: row.get("cpu_brand"),
        tsc_frequency: tsc_frequency as u64,
        start_time: row.get("start_time"),
        start_ticks: row.get("start_ticks"),
        parent_process_id: row.get("parent_process_id"),
    }
}

pub async fn processes_by_name_substring(
    connection: &mut sqlx::AnyConnection,
    filter: &str,
) -> Result<Vec<ProcessInfo>> {
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
) -> Result<ProcessInfo> {
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
) -> Result<Vec<lgn_telemetry_proto::analytics::ProcessInstance>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, 
                exe, 
                username, 
                realname, 
                computer, 
                distro, 
                cpu_brand, 
                tsc_frequency, 
                start_time, 
                start_ticks, 
                parent_process_id,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%cpu%' ) as nb_cpu_blocks,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%log%' ) as nb_log_blocks,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%metric%' ) as nb_metric_blocks
         FROM processes
         ORDER BY start_time DESC
         LIMIT 100;",
    )
    .fetch_all(connection)
    .await?;
    for r in rows {
        let nb_cpu_blocks: i32 = r.get("nb_cpu_blocks");
        let nb_log_blocks: i32 = r.get("nb_log_blocks");
        let nb_metric_blocks: i32 = r.get("nb_metric_blocks");
        let instance = lgn_telemetry_proto::analytics::ProcessInstance {
            process_info: Some(process_from_row(&r)),
            nb_cpu_blocks: nb_cpu_blocks as u32,
            nb_log_blocks: nb_log_blocks as u32,
            nb_metric_blocks: nb_metric_blocks as u32,
        };
        processes.push(instance);
    }
    Ok(processes)
}

pub async fn search_processes(
    connection: &mut sqlx::AnyConnection,
    exe_substr: &str,
) -> Result<Vec<lgn_telemetry_proto::analytics::ProcessInstance>> {
    let mut processes = Vec::new();
    let rows = sqlx::query(
        "SELECT process_id, 
                exe, 
                username, 
                realname, 
                computer, 
                distro, 
                cpu_brand, 
                tsc_frequency, 
                start_time, 
                start_ticks, 
                parent_process_id,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%cpu%' ) as nb_cpu_blocks,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%log%' ) as nb_log_blocks,
                (
                  SELECT count(*)
                  FROM blocks, streams
                  WHERE blocks.stream_id = streams.stream_id
                  AND streams.process_id = processes.process_id
                  AND streams.tags LIKE '%metric%' ) as nb_metric_blocks
         FROM processes
         WHERE exe LIKE ?
         ORDER BY start_time DESC
         LIMIT 100;",
    )
    .bind(format!("%{}%", exe_substr))
    .fetch_all(connection)
    .await?;
    for r in rows {
        let nb_cpu_blocks: i32 = r.get("nb_cpu_blocks");
        let nb_log_blocks: i32 = r.get("nb_log_blocks");
        let nb_metric_blocks: i32 = r.get("nb_metric_blocks");
        let instance = lgn_telemetry_proto::analytics::ProcessInstance {
            process_info: Some(process_from_row(&r)),
            nb_cpu_blocks: nb_cpu_blocks as u32,
            nb_log_blocks: nb_log_blocks as u32,
            nb_metric_blocks: nb_metric_blocks as u32,
        };
        processes.push(instance);
    }
    Ok(processes)
}

pub async fn fetch_child_processes(
    connection: &mut sqlx::AnyConnection,
    parent_process_id: &str,
) -> Result<Vec<ProcessInfo>> {
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
) -> Result<Vec<StreamInfo>> {
    let rows = sqlx::query(
        "SELECT stream_id, process_id, dependencies_metadata, objects_metadata, tags, properties
         FROM streams
         WHERE tags LIKE ?
         AND process_id = ?
         ;",
    )
    .bind(format!("%{}%", tag))
    .bind(process_id)
    .fetch_all(connection)
    .await
    .with_context(|| "fetch_all in find_process_streams_tagged")?;
    let mut res = Vec::new();
    for r in rows {
        let stream_id: String = r.get("stream_id");
        let dependencies_metadata_buffer: Vec<u8> = r.get("dependencies_metadata");
        let dependencies_metadata = lgn_telemetry_proto::telemetry::ContainerMetadata::decode(
            &*dependencies_metadata_buffer,
        )
        .with_context(|| "decoding dependencies metadata")?;
        let objects_metadata_buffer: Vec<u8> = r.get("objects_metadata");
        let objects_metadata =
            lgn_telemetry_proto::telemetry::ContainerMetadata::decode(&*objects_metadata_buffer)
                .with_context(|| "decoding objects metadata")?;
        let tags_str: String = r.get("tags");
        let properties_str: String = r.get("properties");
        let properties: std::collections::HashMap<String, String> =
            serde_json::from_str(&properties_str).unwrap();
        res.push(StreamInfo {
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

pub async fn find_process_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<StreamInfo>> {
    let rows = sqlx::query(
        "SELECT stream_id, process_id, dependencies_metadata, objects_metadata, tags, properties
         FROM streams
         WHERE process_id = ?
         ;",
    )
    .bind(process_id)
    .fetch_all(connection)
    .await
    .with_context(|| "fetch_all in find_process_streams")?;
    let mut res = Vec::new();
    for r in rows {
        let stream_id: String = r.get("stream_id");
        let dependencies_metadata_buffer: Vec<u8> = r.get("dependencies_metadata");
        let dependencies_metadata = lgn_telemetry_proto::telemetry::ContainerMetadata::decode(
            &*dependencies_metadata_buffer,
        )
        .with_context(|| "decoding dependencies metadata")?;
        let objects_metadata_buffer: Vec<u8> = r.get("objects_metadata");
        let objects_metadata =
            lgn_telemetry_proto::telemetry::ContainerMetadata::decode(&*objects_metadata_buffer)
                .with_context(|| "decoding objects metadata")?;
        let tags_str: String = r.get("tags");
        let properties_str: String = r.get("properties");
        let properties: std::collections::HashMap<String, String> =
            serde_json::from_str(&properties_str).unwrap();
        res.push(StreamInfo {
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
) -> Result<Vec<StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "log").await
}

pub async fn find_process_thread_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "cpu").await
}

pub async fn find_process_metrics_streams(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<Vec<StreamInfo>> {
    find_process_streams_tagged(connection, process_id, "metrics").await
}

pub async fn find_stream(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<StreamInfo> {
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
    let dependencies_metadata =
        lgn_telemetry_proto::telemetry::ContainerMetadata::decode(&*dependencies_metadata_buffer)
            .with_context(|| "decoding dependencies metadata")?;
    let objects_metadata_buffer: Vec<u8> = row.get("objects_metadata");
    let objects_metadata =
        lgn_telemetry_proto::telemetry::ContainerMetadata::decode(&*objects_metadata_buffer)
            .with_context(|| "decoding objects metadata")?;
    let tags_str: String = row.get("tags");
    let properties_str: String = row.get("properties");
    let properties: std::collections::HashMap<String, String> =
        serde_json::from_str(&properties_str).unwrap();
    Ok(StreamInfo {
        stream_id: String::from(stream_id),
        process_id: row.get("process_id"),
        dependencies_metadata: Some(dependencies_metadata),
        objects_metadata: Some(objects_metadata),
        tags: tags_str.split(' ').map(ToOwned::to_owned).collect(),
        properties,
    })
}

pub async fn find_block(
    connection: &mut sqlx::AnyConnection,
    block_id: &str,
) -> Result<EncodedBlock> {
    let row = sqlx::query(
        "SELECT stream_id, begin_time, begin_ticks, end_time, end_ticks, nb_objects
         FROM blocks
         WHERE block_id = ?
         ;",
    )
    .bind(block_id)
    .fetch_one(connection)
    .await
    .with_context(|| "find_block")?;

    let block = EncodedBlock {
        block_id: String::from(block_id),
        stream_id: row.get("stream_id"),
        begin_time: row.get("begin_time"),
        begin_ticks: row.get("begin_ticks"),
        end_time: row.get("end_time"),
        end_ticks: row.get("end_ticks"),
        payload: None,
        nb_objects: row.get("nb_objects"),
    };
    Ok(block)
}

pub async fn find_stream_blocks(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
) -> Result<Vec<EncodedBlock>> {
    let blocks = sqlx::query(
        "SELECT block_id, begin_time, begin_ticks, end_time, end_ticks, nb_objects
         FROM blocks
         WHERE stream_id = ?
         ORDER BY begin_time;",
    )
    .bind(stream_id)
    .fetch_all(connection)
    .await
    .with_context(|| "find_stream_blocks")?
    .iter()
    .map(|r| EncodedBlock {
        block_id: r.get("block_id"),
        stream_id: String::from(stream_id),
        begin_time: r.get("begin_time"),
        begin_ticks: r.get("begin_ticks"),
        end_time: r.get("end_time"),
        end_ticks: r.get("end_ticks"),
        payload: None,
        nb_objects: r.get("nb_objects"),
    })
    .collect();
    Ok(blocks)
}

pub async fn find_stream_blocks_in_range(
    connection: &mut sqlx::AnyConnection,
    stream_id: &str,
    begin_time: &str,
    end_time: &str,
) -> Result<Vec<EncodedBlock>> {
    let blocks = sqlx::query(
        "SELECT block_id, begin_time, begin_ticks, end_time, end_ticks, nb_objects
         FROM blocks
         WHERE stream_id = ?
         AND begin_time <= ?
         AND end_time >= ?
         ORDER BY begin_time;",
    )
    .bind(stream_id)
    .bind(end_time)
    .bind(begin_time)
    .fetch_all(connection)
    .await
    .with_context(|| "find_stream_blocks")?
    .iter()
    .map(|r| EncodedBlock {
        block_id: r.get("block_id"),
        stream_id: String::from(stream_id),
        begin_time: r.get("begin_time"),
        begin_ticks: r.get("begin_ticks"),
        end_time: r.get("end_time"),
        end_ticks: r.get("end_ticks"),
        payload: None,
        nb_objects: r.get("nb_objects"),
    })
    .collect();
    Ok(blocks)
}

pub async fn fetch_block_payload(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    block_id: String,
) -> Result<lgn_telemetry_proto::telemetry::BlockPayload> {
    let opt_row = sqlx::query("SELECT payload FROM payloads where block_id = ?;")
        .bind(&block_id)
        .fetch_optional(connection)
        .await
        .with_context(|| format!("Fetching payload of block {}", &block_id))?;

    let buffer: Vec<u8> = if let Some(row) = opt_row {
        row.get("payload")
    } else {
        blob_storage
            .read_blob(&block_id)
            .await
            .with_context(|| "reading block payload from blob storage")?
    };

    let payload = lgn_telemetry_proto::telemetry::BlockPayload::decode(&*buffer)
        .with_context(|| format!("reading payload {}", &block_id))?;
    Ok(payload)
}

fn container_metadata_as_transit_udt_vec(
    value: &ContainerMetadata,
) -> Vec<lgn_tracing_transit::UserDefinedType> {
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
#[span_fn]
pub fn parse_block<F>(
    stream: &StreamInfo,
    payload: &lgn_telemetry_proto::telemetry::BlockPayload,
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

fn format_log_level(level: u32) -> &'static str {
    match level {
        1 => "Error",
        2 => "Warn",
        3 => "Info",
        4 => "Debug",
        5 => "Trace",
        _ => "Unknown",
    }
}

fn log_entry_from_value(val: &Value) -> Option<(i64, String)> {
    if let Value::Object(obj) = val {
        match obj.type_name.as_str() {
            "LogStaticStrEvent" => {
                let time = obj.get::<i64>("time").unwrap();
                let desc = obj.get::<lgn_tracing_transit::Object>("desc").unwrap();
                let level = desc.get::<u32>("level").unwrap();
                let entry = format!(
                    "[{}] {}",
                    format_log_level(level),
                    desc.get::<String>("fmt_str").unwrap()
                );
                Some((time, entry))
            }
            "LogStringEvent" => {
                let time = obj.get::<i64>("time").unwrap();
                let desc = obj.get::<lgn_tracing_transit::Object>("desc").unwrap();
                let level = desc.get::<u32>("level").unwrap();
                let entry = format!(
                    "[{}] {}",
                    format_log_level(level),
                    obj.get::<String>("msg").unwrap()
                );
                Some((time, entry))
            }
            _ => None,
        }
    } else {
        None
    }
}

// find_process_log_entry calls pred(time_ticks,entry_str) with each log entry
// until pred returns Some(x)
pub async fn find_process_log_entry<Res, Predicate: FnMut(i64, String) -> Option<Res>>(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
    mut pred: Predicate,
) -> Result<Option<Res>> {
    let mut found_entry = None;
    for stream in find_process_log_streams(connection, process_id).await? {
        for b in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload =
                fetch_block_payload(connection, blob_storage.clone(), b.block_id.clone()).await?;
            parse_block(&stream, &payload, |val| {
                if let Some((time, msg)) = log_entry_from_value(&val) {
                    if let Some(x) = pred(time, msg) {
                        found_entry = Some(x);
                        return false; //do not continue
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

// for_each_log_entry_in_block calls fun(time_ticks,entry_str) with each log
// entry until fun returns false mad
pub async fn for_each_log_entry_in_block<Predicate: FnMut(i64, String) -> bool>(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    stream: &StreamInfo,
    block: &EncodedBlock,
    mut fun: Predicate,
) -> Result<()> {
    let payload = fetch_block_payload(connection, blob_storage, block.block_id.clone()).await?;
    parse_block(stream, &payload, |val| {
        if let Some((time, msg)) = log_entry_from_value(&val) {
            if !fun(time, msg) {
                return false; //do not continue
            }
        }
        true //continue
    })?;
    Ok(())
}

pub async fn for_each_process_log_entry<ProcessLogEntry: FnMut(i64, String)>(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
    mut process_log_entry: ProcessLogEntry,
) -> Result<()> {
    find_process_log_entry(connection, blob_storage, process_id, |time, entry| {
        process_log_entry(time, entry);
        let nothing: Option<()> = None;
        nothing //continue searching
    })
    .await?;
    Ok(())
}

pub async fn for_each_process_metric<ProcessMetric: FnMut(lgn_tracing_transit::Object)>(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
    mut process_metric: ProcessMetric,
) -> Result<()> {
    for stream in find_process_metrics_streams(connection, process_id).await? {
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload =
                fetch_block_payload(connection, blob_storage.clone(), block.block_id.clone())
                    .await?;
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
    root: &ProcessInfo,
    rec_level: u16,
    fun: F,
) -> Result<()>
where
    F: Fn(&ProcessInfo, u16) + std::marker::Send + Clone,
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
    pub use crate::find_block;
    pub use crate::find_process;
    pub use crate::find_process_log_entry;
    pub use crate::find_process_log_streams;
    pub use crate::find_process_metrics_streams;
    pub use crate::find_process_streams;
    pub use crate::find_process_thread_streams;
    pub use crate::find_stream_blocks;
    pub use crate::find_stream_blocks_in_range;
    pub use crate::for_each_log_entry_in_block;
    pub use crate::for_each_process_in_tree;
    pub use crate::for_each_process_log_entry;
    pub use crate::for_each_process_metric;
    pub use crate::parse_block;
    pub use crate::processes_by_name_substring;
    pub use crate::search_processes;
}
