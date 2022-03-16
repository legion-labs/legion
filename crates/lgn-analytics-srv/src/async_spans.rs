use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::{
    AsyncSpansReply, BlockAsyncEventsStatReply, ScopeDesc, Span, SpanTrack,
};
use lgn_tracing::prelude::*;
use lgn_tracing_transit::Object;
use std::collections::HashMap;

use crate::{
    scope::{compute_scope_hash, ScopeHashMap},
    thread_block_processor::{parse_thread_block, ThreadBlockProcessor},
};

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
#[span_fn]
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

fn ranges_overlap(begin_a: f64, end_a: f64, begin_b: f64, end_b: f64) -> bool {
    begin_a <= end_b && begin_b <= end_a
}

#[derive(Debug)]
struct BeginSpan {
    time_ms: f64,
}

#[derive(Debug)]
struct EndSpan {
    time_ms: f64,
}

#[derive(Debug)]
enum SpanEvent {
    Begin(BeginSpan),
    End(EndSpan),
}

struct AsyncSpanBuilder {
    begin_section_ms: f64,
    end_section_ms: f64,
    unmatched_events: HashMap<u64, SpanEvent>,
    complete_spans: Vec<Span>,
    scopes: ScopeHashMap,
    convert_ticks: ConvertTicks,
}

impl AsyncSpanBuilder {
    fn new(begin_section_ms: f64, end_section_ms: f64, convert_ticks: ConvertTicks) -> Self {
        Self {
            begin_section_ms,
            end_section_ms,
            unmatched_events: HashMap::new(),
            complete_spans: Vec::new(),
            scopes: ScopeHashMap::new(),
            convert_ticks,
        }
    }

    fn record_scope_desc(&mut self, hash: u32, name: &str) {
        self.scopes.entry(hash).or_insert_with(|| ScopeDesc {
            name: name.to_owned(),
            filename: "".to_string(),
            line: 0,
            hash,
        });
    }

    fn record_span(&mut self, begin_ms: f64, end_ms: f64, scope: &Arc<Object>) -> Result<()> {
        if ranges_overlap(self.begin_section_ms, self.end_section_ms, begin_ms, end_ms) {
            let scope_name = scope.get::<Arc<String>>("name")?;
            let scope_hash = compute_scope_hash(&scope_name);
            self.record_scope_desc(scope_hash, &scope_name);
            self.complete_spans.push(Span {
                scope_hash,
                begin_ms,
                end_ms,
                alpha: 255,
            });
        }
        Ok(())
    }

    #[span_fn]
    fn finish(self) -> (Vec<Span>, ScopeHashMap) {
        //todo: complete unmatched events
        (self.complete_spans, self.scopes)
    }
}

impl ThreadBlockProcessor for AsyncSpanBuilder {
    fn on_begin_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }

    fn on_end_thread_scope(&mut self, _scope: Arc<Object>, _ts: i64) -> Result<()> {
        Ok(())
    }

    fn on_begin_async_scope(&mut self, span_id: u64, scope: Arc<Object>, ts: i64) -> Result<()> {
        let time_ms = self.convert_ticks.get_time(ts);
        if let Some(evt) = self.unmatched_events.remove(&span_id) {
            match evt {
                SpanEvent::Begin(begin_span) => {
                    anyhow::bail!(
                        "duplicate begin event for span id {}: {:?}",
                        span_id,
                        begin_span
                    );
                }
                SpanEvent::End(end_event) => {
                    self.record_span(time_ms, end_event.time_ms, &scope)?;
                }
            }
        } else {
            self.unmatched_events
                .insert(span_id, SpanEvent::Begin(BeginSpan { time_ms }));
        }
        Ok(())
    }

    fn on_end_async_scope(&mut self, span_id: u64, scope: Arc<Object>, ts: i64) -> Result<()> {
        let time_ms = self.convert_ticks.get_time(ts);
        if let Some(evt) = self.unmatched_events.remove(&span_id) {
            match evt {
                SpanEvent::End(end_span) => {
                    anyhow::bail!(
                        "duplicate end event for span id {}: {:?}",
                        span_id,
                        end_span
                    );
                }
                SpanEvent::Begin(begin_span) => {
                    self.record_span(begin_span.time_ms, time_ms, &scope)?;
                }
            }
        } else {
            self.unmatched_events
                .insert(span_id, SpanEvent::End(EndSpan { time_ms }));
        }
        Ok(())
    }
}

fn is_track_available(track: &[Span], time: f64) -> bool {
    if let Some(last) = track.last() {
        last.end_ms <= time
    } else {
        true
    }
}

#[span_fn]
fn get_available_track(tracks: &mut Vec<SpanTrack>, time: f64) -> usize {
    for (index, track) in tracks.iter().enumerate() {
        if is_track_available(&track.spans, time) {
            return index;
        }
    }
    tracks.push(SpanTrack { spans: vec![] });
    tracks.len() - 1
}

#[span_fn]
fn layout_spans(spans: Vec<Span>) -> Vec<SpanTrack> {
    let mut tracks = vec![];
    for span in spans {
        let index = get_available_track(&mut tracks, span.begin_ms);
        tracks[index].spans.push(span);
    }
    tracks
}

#[allow(clippy::cast_lossless)]
#[span_fn]
pub async fn compute_async_spans(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    section_sequence_number: i32,
    section_lod: u32,
    block_ids: Vec<String>,
) -> Result<AsyncSpansReply> {
    if section_lod != 0 {
        anyhow::bail!("async lods not implemented");
    }
    let section_width_ms = 1000.0;
    let begin_section_ms = section_sequence_number as f64 * section_width_ms;
    let end_section_ms = begin_section_ms + section_width_ms;
    if block_ids.is_empty() {
        return Ok(AsyncSpansReply {
            section_sequence_number,
            section_lod,
            tracks: vec![],
            scopes: ScopeHashMap::new(),
        });
    }
    let process = find_block_process(connection, &block_ids[0]).await?;
    let mut builder = AsyncSpanBuilder::new(
        begin_section_ms,
        end_section_ms,
        ConvertTicks::new(&process),
    );
    for block_id in &block_ids {
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
    let (mut spans, scopes) = builder.finish();
    spans.sort_by(|a, b| a.begin_ms.partial_cmp(&b.begin_ms).unwrap());
    let tracks = layout_spans(spans);
    let reply = AsyncSpansReply {
        section_sequence_number,
        section_lod,
        tracks,
        scopes,
    };
    Ok(reply)
}
