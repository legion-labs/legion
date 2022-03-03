use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::BlockAsyncEventsStatReply;
use lgn_tracing::warn;
use lgn_tracing_transit::Value;

async fn parse_thread_block(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    stream: &lgn_telemetry_sink::StreamInfo,
    block_id: String,
    stats: &mut BlockAsyncEventsStatReply,
) -> Result<()> {
    let payload = fetch_block_payload(connection, blob_storage, block_id).await?;
    parse_block(stream, &payload, |val| {
        if let Value::Object(obj) = val {
            match obj.type_name.as_str() {
                "BeginThreadSpanEvent" | "EndThreadSpanEvent" => {}
                "BeginAsyncSpanEvent" | "EndAsyncSpanEvent" => {
                    stats.nb_events += 1;
                }
                event_type => {
                    warn!("unknown event type {}", event_type);
                }
            }
        }
        true //continue
    })?;
    Ok(())
}

pub async fn compute_block_async_stats(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    _process: lgn_telemetry_proto::telemetry::Process,
    stream: lgn_telemetry_sink::StreamInfo,
    block_id: String,
) -> Result<BlockAsyncEventsStatReply> {
    // let ts_offset = process.start_ticks;
    // let inv_tsc_frequency = get_process_tick_length_ms(&process);
    let mut stats = BlockAsyncEventsStatReply {
        block_id: block_id.clone(),
        begin_ms: f64::MAX,
        end_ms: f64::MIN,
        nb_events: 0,
    };
    parse_thread_block(connection, blob_storage, &stream, block_id, &mut stats).await?;
    Ok(stats)
}
