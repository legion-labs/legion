use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::{fetch_block_payload, parse_block};
use lgn_blob_storage::BlobStorage;
use lgn_tracing::warn;
use lgn_tracing_transit::{Object, Value};

pub trait ThreadBlockProcessor {
    fn on_begin_thread_scope(&mut self, scope_name: &str, ts: i64);
    fn on_end_thread_scope(&mut self, scope_name: &str, ts: i64);
    fn on_begin_async_scope(&mut self, span_id: u64, scope_name: &str, ts: i64);
    fn on_end_async_scope(&mut self, span_id: u64, scope_name: &str, ts: i64);
}

fn on_thread_event<F>(obj: &lgn_tracing_transit::Object, mut fun: F) -> Result<()>
where
    F: FnMut(&str, i64),
{
    let tick = obj.get::<i64>("time")?;
    let scope = obj.get::<Arc<Object>>("thread_span_desc")?;
    let name = scope.get::<Arc<String>>("name")?;
    fun(&*name, tick);
    Ok(())
}

fn on_async_thread_event<F>(obj: &lgn_tracing_transit::Object, mut fun: F) -> Result<()>
where
    F: FnMut(u64, &str, i64),
{
    let tick = obj.get::<i64>("time")?;
    let span_id = obj.get::<u64>("span_id")?;
    let scope = obj.get::<Arc<Object>>("span_desc")?;
    let name = scope.get::<Arc<String>>("name")?;
    fun(span_id, &*name, tick);
    Ok(())
}

pub async fn parse_thread_block<Proc: ThreadBlockProcessor>(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    stream: &lgn_telemetry_sink::StreamInfo,
    block_id: String,
    processor: &mut Proc,
) -> Result<()> {
    let payload = fetch_block_payload(connection, blob_storage, block_id).await?;
    parse_block(stream, &payload, |val| {
        if let Value::Object(obj) = val {
            match obj.type_name.as_str() {
                "BeginThreadSpanEvent" => {
                    if let Err(e) = on_thread_event(&obj, |name, ts| {
                        processor.on_begin_thread_scope(name, ts);
                    }) {
                        warn!("Error reading BeginThreadSpanEvent: {:?}", e);
                    }
                }
                "EndThreadSpanEvent" => {
                    if let Err(e) = on_thread_event(&obj, |name, ts| {
                        processor.on_end_thread_scope(name, ts);
                    }) {
                        warn!("Error reading EndThreadSpanEvent: {:?}", e);
                    }
                }
                "BeginAsyncSpanEvent" => {
                    if let Err(e) = on_async_thread_event(&obj, |id, name, ts| {
                        processor.on_begin_async_scope(id, name, ts);
                    }) {
                        warn!("Error reading BeginAsyncSpanEvent: {:?}", e);
                    }
                }
                "EndAsyncSpanEvent" => {
                    if let Err(e) = on_async_thread_event(&obj, |id, name, ts| {
                        processor.on_end_async_scope(id, name, ts);
                    }) {
                        warn!("Error reading BeginAsyncSpanEvent: {:?}", e);
                    }
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
