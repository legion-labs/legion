use anyhow::*;
use legion_analytics::*;
use std::path::Path;
use transit::*;

pub async fn print_process_thread_events(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for stream in find_process_thread_streams(connection, process_id).await? {
        println!("stream {}", stream.stream_id);
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            println!("block {}", block.block_id);
            let payload = fetch_block_payload(connection, data_path, &block.block_id).await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    let time = obj.get::<u64>("time").unwrap();
                    let scope = obj.get::<Object>("scope").unwrap();
                    let name = scope.get::<String>("name").unwrap();
                    let filename = scope.get::<String>("filename").unwrap();
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
pub async fn print_chrome_trace(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    let process_info = find_process(connection, String::from(process_id)).await?;
    let inv_tsc_frequency = 1000.0 * 1000.0 / process_info.tsc_frequency as f64;
    let process_start = process_info.start_ticks;
    let mut events = json::array![];
    for stream in find_process_thread_streams(connection, process_id).await? {
        let system_thread_id = &stream.properties["thread-id"];
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload = fetch_block_payload(connection, data_path, &block.block_id).await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    let phase = match obj.type_name.as_str() {
                        "BeginScopeEvent" => "B",
                        "EndScopeEvent" => "E",
                        _ => panic!("unknown event type {}", obj.type_name),
                    };
                    let tick = obj.get::<u64>("time").unwrap();
                    let time = format!("{}", (tick - process_start) as f64 * inv_tsc_frequency);
                    let scope = obj.get::<Object>("scope").unwrap();
                    let name = scope.get::<String>("name").unwrap();
                    let event = json::object! {
                        name: name,
                        cat: "PERF",
                        ph: phase,
                        pid: process_info.process_id.clone(),
                        tid: system_thread_id.clone(),
                        ts: time,

                    };
                    events.push(event).unwrap();
                }
                true //continue
            })?;
        }
    }

    let trace_document = json::object! {
        traceEvents: events,
        displayTimeUnit: "ms",
    };

    println!("{}", trace_document.dump());
    Ok(())
}
