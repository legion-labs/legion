use std::sync::Arc;

use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_blob_storage::BlobStorage;
use lgn_tracing_transit::prelude::*;

pub async fn print_process_thread_events(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
) -> Result<()> {
    for stream in find_process_thread_streams(connection, process_id).await? {
        println!("stream {}", stream.stream_id);
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            println!("block {}", block.block_id);
            let payload =
                fetch_block_payload(connection, blob_storage.clone(), block.block_id.clone())
                    .await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    let time = obj.get::<u64>("time").unwrap();
                    let scope = obj.get::<Object>("thread_span_desc").unwrap();
                    let name = scope.get::<String>("name").unwrap();
                    let filename = scope.get::<String>("file").unwrap();
                    let line = scope.get::<u32>("line").unwrap();
                    println!("{} {} {} {}:{}", time, obj.type_name, name, filename, line);
                }
                true //continue
            })?;
            println!();
        }
        println!();
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
async fn extract_process_thread_events(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_info: &lgn_telemetry_sink::ProcessInfo,
    ts_offset: i64,
    inv_tsc_frequency: f64,
) -> Result<json::Array> {
    let mut events = json::Array::new();
    let process_id = &process_info.process_id;
    for stream in find_process_thread_streams(connection, process_id).await? {
        let system_thread_id = &stream.properties["thread-id"];
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload =
                fetch_block_payload(connection, blob_storage.clone(), block.block_id.clone())
                    .await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    let phase = match obj.type_name.as_str() {
                        "BeginScopeEvent" => "B",
                        "EndScopeEvent" => "E",
                        _ => panic!("unknown event type {}", obj.type_name),
                    };
                    let tick = obj.get::<i64>("time").unwrap();
                    let time = format!("{}", (tick - ts_offset) as f64 * inv_tsc_frequency);
                    let scope = obj.get::<Object>("scope").unwrap();
                    let name = scope.get::<String>("name").unwrap();
                    let event = json::object! {
                        name: name,
                        cat: "PERF",
                        ph: phase,
                        pid: process_id.clone(),
                        tid: system_thread_id.clone(),
                        ts: time,

                    };
                    events.push(event);
                }
                true //continue
            })?;
        }
    }
    Ok(events)
}

#[allow(clippy::cast_precision_loss)]
pub async fn print_chrome_trace(
    pool: &sqlx::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
) -> Result<()> {
    let mut connection = pool.acquire().await?;
    let root_process_info = find_process(&mut connection, process_id).await?;

    let (tx, rx) = std::sync::mpsc::channel();

    let inv_tsc_frequency = 1000.0 * get_process_tick_length_ms(&root_process_info);
    let root_process_start = root_process_info.start_ticks;
    let mut events = json::Array::new();

    for_each_process_in_tree(
        pool,
        &root_process_info,
        0,
        move |process_info, _rec_level| {
            tx.send(process_info.clone()).unwrap();
        },
    )
    .await
    .with_context(|| "print_chrome_trace")?;

    while let Ok(child_process_info) = rx.recv() {
        assert_eq!(
            root_process_info.tsc_frequency,
            child_process_info.tsc_frequency
        );
        let mut child_events = extract_process_thread_events(
            &mut connection,
            blob_storage.clone(),
            &child_process_info,
            root_process_start,
            inv_tsc_frequency,
        )
        .await?;
        events.append(&mut child_events);
    }

    let trace_document = json::object! {
        traceEvents: events,
        displayTimeUnit: "ms",
    };

    println!("{}", trace_document.dump());
    Ok(())
}
