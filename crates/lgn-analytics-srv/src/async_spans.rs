use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::{find_block_stream, get_process_tick_length_ms};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::{AsyncSpansReply, BlockAsyncEventsStatReply, Span};
use lgn_tracing_transit::Object;
use std::collections::HashMap;

use crate::thread_block_processor::{parse_thread_block, ThreadBlockProcessor};

struct StatsProcessor {
    process_start_ts: i64,
    min_ts: i64,
    max_ts: i64,
    nb_events: u64,
}

impl StatsProcessor {
    fn new(process_start_ts: i64) -> Self {
        Self {
            process_start_ts,
            min_ts: i64::MAX,
            max_ts: i64::MIN,
            nb_events: 0,
        }
    }
}

impl ThreadBlockProcessor for StatsProcessor {
    fn on_begin_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }
    fn on_end_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }

    fn on_begin_async_scope(&mut self, _span_id: u64, _scope: Arc<Object>, ts: i64) -> Result<()> {
        let relative_ts = ts - self.process_start_ts;
        self.min_ts = self.min_ts.min(relative_ts);
        self.max_ts = self.max_ts.max(relative_ts);
        self.nb_events += 1;
        Ok(())
    }

    fn on_end_async_scope(&mut self, _span_id: u64, _scope: Arc<Object>, ts: i64) -> Result<()> {
        let relative_ts = ts - self.process_start_ts;
        self.min_ts = self.min_ts.min(relative_ts);
        self.max_ts = self.max_ts.max(relative_ts);
        self.nb_events += 1;
        Ok(())
    }
}

#[allow(clippy::cast_precision_loss)]
pub async fn compute_block_async_stats(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process: lgn_telemetry_proto::telemetry::Process,
    stream: lgn_telemetry_sink::StreamInfo,
    block_id: String,
) -> Result<BlockAsyncEventsStatReply> {
    let inv_tsc_frequency = get_process_tick_length_ms(&process);
    let mut processor = StatsProcessor::new(process.start_ticks);
    parse_thread_block(
        connection,
        blob_storage,
        &stream,
        block_id.clone(),
        &mut processor,
    )
    .await?;
    Ok(BlockAsyncEventsStatReply {
        block_id,
        begin_ms: processor.min_ts as f64 * inv_tsc_frequency,
        end_ms: processor.max_ts as f64 * inv_tsc_frequency,
        nb_events: processor.nb_events,
    })
}

#[derive(Debug)]
struct BeginSpan {}

#[derive(Debug)]
struct EndSpan {}

#[derive(Debug)]
enum SpanEvent {
    Begin(BeginSpan),
    End(EndSpan),
}

struct AsyncSpanBuilder {
    unmatched_events: HashMap<u64, SpanEvent>,
    complete_spans: Vec<Span>,
}

impl AsyncSpanBuilder {
    fn new() -> Self {
        Self {
            unmatched_events: HashMap::new(),
            complete_spans: Vec::new(),
        }
    }
}

impl ThreadBlockProcessor for AsyncSpanBuilder {
    fn on_begin_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }

    fn on_end_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }

    fn on_begin_async_scope(&mut self, span_id: u64, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        if let Some(evt) = self.unmatched_events.remove(&span_id) {
            match evt {
                SpanEvent::Begin(begin_span) => {
                    anyhow::bail!(
                        "duplicate begin event for span id {}: {:?}",
                        span_id,
                        begin_span
                    );
                }
                SpanEvent::End(_end_event) => {
                    self.complete_spans.push(Span {
                        scope_hash: 0,
                        begin_ms: 0.0,
                        end_ms: 0.0,
                        alpha: 255,
                    });
                }
            }
        } else {
            self.unmatched_events
                .insert(span_id, SpanEvent::Begin(BeginSpan {}));
        }
        Ok(())
    }

    fn on_end_async_scope(&mut self, span_id: u64, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        if let Some(evt) = self.unmatched_events.remove(&span_id) {
            match evt {
                SpanEvent::End(end_span) => {
                    anyhow::bail!(
                        "duplicate end event for span id {}: {:?}",
                        span_id,
                        end_span
                    );
                }
                SpanEvent::Begin(_begin_span) => {
                    self.complete_spans.push(Span {
                        scope_hash: 0,
                        begin_ms: 0.0,
                        end_ms: 0.0,
                        alpha: 255,
                    });
                }
            }
        } else {
            self.unmatched_events
                .insert(span_id, SpanEvent::End(EndSpan {}));
        }
        Ok(())
    }
}

pub async fn compute_async_spans(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    section_sequence_number: i32,
    section_lod: u32,
    block_ids: Vec<String>,
) -> Result<AsyncSpansReply> {
    for block_id in &block_ids {
        let mut builder = AsyncSpanBuilder::new();
        let stream = find_block_stream(connection, block_id).await?;
        parse_thread_block(
            connection,
            blob_storage.clone(),
            &stream,
            block_id.clone(),
            &mut builder,
        )
        .await?;
    }
    let tracks = vec![];
    let reply = AsyncSpansReply {
        section_sequence_number,
        section_lod,
        tracks,
    };
    Ok(reply)
}
